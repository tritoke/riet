mod point;
pub use point::Point;

mod color;
pub use color::Color;

mod color_block;
pub use color_block::ColorBlock;

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

use image::RgbImage;

use crate::interpreter::Interpreter;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DirectionPointer {
    #[default]
    Right,
    Left,
    Down,
    Up,
}

impl DirectionPointer {
    pub fn rotate_clockwise(self) -> Self {
        use DirectionPointer as DP;

        #[rustfmt::skip]
        match self {
            DP::Right => DP::Down,
            DP::Down  => DP::Left,
            DP::Left  => DP::Up,
            DP::Up    => DP::Right,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CodelChooser {
    #[default]
    Left,
    Right,
}

impl CodelChooser {
    pub fn toggle(self) -> Self {
        use CodelChooser as CC;

        #[rustfmt::skip]
        match self {
            CC::Right => CC::Left,
            CC::Left  => CC::Right,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Program {
    codels: Vec<Color>,
    blocks: HashMap<Point, Rc<RefCell<ColorBlock>>>,
    rows: u32,
    cols: u32,
}

impl Program {
    fn new(rows: u32, cols: u32) -> Self {
        Self {
            codels: vec![Color::White; (rows * cols) as usize],
            blocks: Default::default(),
            rows,
            cols,
        }
    }

    pub fn rows(&self) -> &u32 {
        &self.rows
    }

    pub fn cols(&self) -> &u32 {
        &self.cols
    }

    /// Get a reference to a codel in a program
    pub fn get_codel(&self, row: u32, col: u32) -> Option<&Color> {
        if row < self.rows && col < self.cols {
            self.codels.get((row * self.cols + col) as usize)
        } else {
            None
        }
    }

    /// get a mutable reference to a codel in a program
    fn get_codel_mut(&mut self, row: u32, col: u32) -> Option<&mut Color> {
        if row < self.rows && col < self.cols {
            self.codels.get_mut((row * self.cols + col) as usize)
        } else {
            None
        }
    }

    /// Get a reference to a color block in a program
    pub fn get_color_block(&self, point: &Point) -> Option<Ref<ColorBlock>> {
        self.blocks.get(point).map(|b| (**b).borrow())
    }

    /// Get a reference to a color block in a program
    pub fn get_color_block_mut(&self, point: &Point) -> Option<RefMut<ColorBlock>> {
        self.blocks.get(point).map(|b| (**b).borrow_mut())
    }

    /// Construct a new piet program from an imagebuffer containing a piet image.
    pub fn new_from_imagebuffer(img: &RgbImage, codel_size: u32) -> Self {
        let mut program = if codel_size == 1 {
            // special case a codel size of 1 for efficiency
            Self {
                codels: img.pixels().map(Color::from_rgb8).collect(),
                blocks: Default::default(),
                rows: img.height(),
                cols: img.width(),
            }
        } else {
            let cols = img.width() / codel_size;
            let rows = img.height() / codel_size;

            let mut program = Self::new(rows, cols);

            for row in 0..rows {
                for col in 0..cols {
                    // vote for the color of the codel
                    let mut votes: HashMap<Color, u32> = HashMap::new();

                    let tl_x = col * codel_size;
                    let tl_y = row * codel_size;

                    for x in tl_x..tl_x + codel_size {
                        for y in tl_y..tl_y + codel_size {
                            let color = Color::from_rgb8(img.get_pixel(x, y));
                            *votes.entry(color).or_insert(0) += 1;
                        }
                    }

                    // the colour of the codel is the one with the most votes
                    if let Some((codel_color, _)) =
                        votes.into_iter().max_by_key(|&(_, votes)| votes)
                    {
                        let codel = program.get_codel_mut(row, col).unwrap();
                        *codel = codel_color;
                    }
                }
            }

            program
        };

        // we now need to fill in the code blocks
        for col in 0..program.cols {
            for row in 0..program.rows {
                let codel_color = *program.get_codel(row, col).unwrap();

                // create a new color block for ourselves
                program.blocks.insert(
                    Point(row, col),
                    Rc::new(RefCell::new(ColorBlock::new(codel_color, row, col))),
                );

                // represent valid neighbours by a pair of Some values
                let neighbours = [
                    (row.checked_sub(1), Some(col)),
                    (
                        if row + 1 < program.rows {
                            Some(row + 1)
                        } else {
                            None
                        },
                        Some(col),
                    ),
                    (Some(row), col.checked_sub(1)),
                    (
                        Some(row),
                        if col + 1 < program.cols {
                            Some(col + 1)
                        } else {
                            None
                        },
                    ),
                ];

                // check if any neighbours are the same colour
                for neighbour in neighbours {
                    let point = if let (Some(r), Some(c)) = neighbour {
                        Point(r, c)
                    } else {
                        continue;
                    };

                    if let Some(neigh_block) = program.blocks.get(&point) {
                        let neigh_color: Color = (**neigh_block).borrow().color();

                        if codel_color == neigh_color {
                            program.merge_color_blocks(&Point(row, col), &point);
                        }
                    }
                }
            }
        }

        program
    }

    /// merge two color blocks together
    fn merge_color_blocks(&mut self, point1: &Point, point2: &Point) {
        // steps:
        // -1. check we are not merging a block into itself.
        // 0. determine which of the points is bigger.
        // 1. add all the points in the area of the smaller one to the area of the bigger one.
        // 2. go through the edges of the smaller one and determine which, if any, are more extreme than those of the parent color block.
        // 3. ensure entries in the smaller block point to the new block.

        // -1. check we are not merging a block into itself.
        if self.blocks.get(point1).eq(&self.blocks.get(point2)) {
            return;
        }

        // 0. determine which of the points is bigger.
        let (bigger_point, smaller_point) = {
            let block1 = self
                .blocks
                .get(point1)
                .expect("Tried to merge a non-existant block.");
            let block2 = self
                .blocks
                .get(point2)
                .expect("Tried to merge a non-existant block.");

            if block1 < block2 {
                (point2, point1)
            } else {
                (point1, point2)
            }
        };

        // declare a new scope for mutating the program
        {
            let mut bigger = self.get_color_block_mut(bigger_point).unwrap();
            let smaller = self.get_color_block(smaller_point).unwrap();

            // 1. extend the bigger area with the points from the smaller one
            bigger.area_mut().extend(smaller.area().iter());

            // 2. go through the edges of the smaller one and determine which, if any, are more extreme than those of the parent color block.
            for point in smaller.edges().values() {
                bigger.add_codel(*point.row(), *point.col());
            }
        }

        // 3. ensure entries in the smaller block point to the bigger block.
        let bigger_block = self.blocks.get(bigger_point).unwrap().clone();
        let smaller_area = self.get_color_block(smaller_point).unwrap().area().clone();

        for point in smaller_area {
            self.blocks.insert(point, bigger_block.clone());
        }
    }

    pub fn into_interpreter(self) -> Interpreter {
        Interpreter::new(self)
    }

    /// Save the codels to an image, with each codel represented with one pixel
    #[allow(dead_code)]
    pub fn save_codels(&self, path: &str) -> anyhow::Result<()> {
        let mut colours = vec![];

        for codel in self.codels.iter() {
            let image::Rgb([r, g, b]) = codel.to_rgb8();

            colours.push(r);
            colours.push(g);
            colours.push(b);
        }

        let buf: image::RgbImage =
            image::ImageBuffer::from_vec(*self.cols(), *self.rows(), colours)
                .ok_or_else(|| anyhow::anyhow!("Failed to encode codels as pixels."))?;

        buf.save(path)?;

        Ok(())
    }
}
