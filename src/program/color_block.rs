use super::{CodelChooser as CC, Color, DirectionPointer as DP, Point};
use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ColorBlock {
    color: Color,
    area: HashSet<Point>,
    edges: HashMap<(DP, CC), Point>,
}

impl Ord for ColorBlock {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.area.len().cmp(&other.area.len())
    }
}

impl PartialOrd for ColorBlock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ColorBlock {
    fn eq(&self, other: &Self) -> bool {
        self.area == other.area
    }
}

impl Eq for ColorBlock {}

impl ColorBlock {
    pub(super) fn new(color: Color, row: u32, col: u32) -> Self {
        let mut cb = Self {
            color,
            area: Default::default(),
            edges: Default::default(),
        };

        cb.add_codel(row, col);

        cb
    }

    /// Add a codel to the colour block
    pub(super) fn add_codel(&mut self, row: u32, col: u32) {
        let point = Point(row, col);
        self.area.insert(point);

        if self.edges.is_empty() {
            for dp in [DP::Up, DP::Down, DP::Left, DP::Right] {
                for cc in [CC::Left, CC::Right] {
                    self.edges.insert((dp, cc), point);
                }
            }
        } else {
            for ((dp, cc), point) in self.edges.iter_mut() {
                #[rustfmt::skip]
                match (dp, cc, col.cmp(point.col()), row.cmp(point.row())) {
                    (DP::Up,    CC::Left,  Less,    Equal)
                  | (DP::Up,    CC::Right, Greater, Equal)

                  | (DP::Down,  CC::Left,  Greater, Equal)
                  | (DP::Down,  CC::Right, Less,    Equal)

                  | (DP::Right, CC::Left,  Equal,   Less)
                  | (DP::Right, CC::Right, Equal,   Greater)

                  | (DP::Left,  CC::Left,  Equal,   Greater)
                  | (DP::Left,  CC::Right, Equal,   Less)

                  | (DP::Down,  _,         _,       Greater)
                  | (DP::Up,    _,         _,       Less)

                  | (DP::Left,  _,         Less,    _)
                  | (DP::Right, _,         Greater, _)
                    => {
                        *point.row_mut() = row;
                        *point.col_mut() = col;
                    }

                    _ => {}
                }
            }
        }
    }
}

#[allow(dead_code)]
impl ColorBlock {
    pub fn color(&self) -> Color {
        self.color
    }

    pub(super) fn area(&self) -> &HashSet<Point> {
        &self.area
    }

    pub(super) fn area_mut(&mut self) -> &mut HashSet<Point> {
        &mut self.area
    }

    pub(super) fn edges(&self) -> &HashMap<(DP, CC), Point> {
        &self.edges
    }

    pub(super) fn edges_mut(&mut self) -> &mut HashMap<(DP, CC), Point> {
        &mut self.edges
    }

    pub fn num_codels(&self) -> usize {
        self.area.len()
    }

    pub fn edge(&self, dp: DP, cc: CC) -> Point {
        self.edges[&(dp, cc)]
    }
}
