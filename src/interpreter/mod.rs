use crate::program::{CodelChooser, Color, DirectionPointer, Point, Program};

use std::io::{self, prelude::*};

mod stack;
use stack::Stack;

use anyhow::bail;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::Zero;

#[derive(Debug, Default, Clone)]
struct PietState {
    dp: DirectionPointer,
    cc: CodelChooser,
    curr_codel: Point,
    stack: Stack,
    escape_attempts: u32,
}

#[derive(Debug)]
pub struct Interpreter {
    program: Program,
    state: PietState,
    step_no: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum IoType {
    Char,
    Number,
}

impl Interpreter {
    pub fn new(program: Program) -> Self {
        Self {
            program,
            state: Default::default(),
            step_no: 0,
        }
    }

    pub fn step(&mut self) -> anyhow::Result<()> {
        let edge_codel;
        let block_value;
        {
            let cb = self
                .program
                .get_color_block(&self.state.curr_codel)
                .unwrap();

            edge_codel = cb.edge(self.state.dp, self.state.cc);
            block_value = cb.num_codels();
        }

        // handle the case when the codel is white
        let next_codel = edge_codel.next_in_direction(self.state.dp, &self.program);
        let move_off_or_black = if let Some(codel) = next_codel {
            self.program
                .get_codel(*codel.row(), *codel.col())
                .contains(&&Color::Black)
        } else {
            true
        };

        if move_off_or_black {
            if self.state.escape_attempts >= 8 {
                std::process::exit(0);
            }

            self.state.escape_attempts += 1;

            if self.state.escape_attempts % 2 == 0 {
                self.state.dp = self.state.dp.rotate_clockwise();
            } else {
                self.state.cc = self.state.cc.toggle();
            }
        } else {
            self.state.escape_attempts = 0;

            let curr = self.state.curr_codel;
            let next = next_codel.unwrap();

            let curr_color = *self.program.get_codel(*curr.row(), *curr.col()).unwrap();
            let next_color = *self.program.get_codel(*next.row(), *next.col()).unwrap();

            let hue_change = curr_color.hue_change(&next_color);
            let lightness_change = curr_color.lightness_change(&next_color);

            trace!("step {:}  {:?} {:?}|{:?} {:?} -> {:?} {:?}|{:?} {:?}",
                self.step_no,
                curr,
                self.state.dp,
                self.state.cc,
                curr_color,
                next,
                self.state.dp,
                self.state.cc,
                next_color,
            );

            if let (Some(hc), Some(lc)) = (hue_change, lightness_change) {
                // Hue change	None	    1 Darker	2 Darker
                // None	 	                push        pop
                // 1 Step	    add	        subtract	multiply
                // 2 Steps	    divide	    mod	        not
                // 3 Steps	    greater	    pointer	    switch
                // 4 Steps	    duplicate	roll	    in(number)
                // 5 Steps	    in(char)	out(number)	out(char)

                #[rustfmt::skip]
                match (hc, lc) {
                    (0, 0) => {},
                    (0, 1) => { self.push(block_value); },
                    (0, 2) => { self.pop(); },

                    (1, 0) => { self.add(); },
                    (1, 1) => { self.subtract(); },
                    (1, 2) => { self.multiply(); },

                    (2, 0) => { self.divide(); },
                    (2, 1) => { self.r#mod(); },
                    (2, 2) => { self.not(); },

                    (3, 0) => { self.greater(); },
                    (3, 1) => { self.pointer(); },
                    (3, 2) => { self.switch(); },

                    (4, 0) => { self.duplicate(); },
                    (4, 1) => { todo!("roll") },
                    (4, 2) => { todo!("in(number)") },

                    (5, 0) => { todo!("in(char)") },
                    (5, 1) => { self.out(IoType::Number); },
                    (5, 2) => { self.out(IoType::Char); },

                    (_, _) => { bail!("Unknown hue/lightness change: (lc:{:?}, hc:{:?})", lc, hc) }
                }
            }

            self.step_no += 1;
            self.state.curr_codel = next;
        }

        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<!> {
        loop {
            self.step()?
        }
    }

    fn push(&mut self, v: impl Into<BigInt> + std::fmt::Debug) {
        trace!("action: push, value {:?}", v);

        self.state.stack.push(v.into());
    }

    fn pop(&mut self) {
        trace!("action: pop");

        self.state.stack.pop();
    }

    fn add(&mut self) -> Option<()> {
        trace!("action: add");

        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop()?;
            let b = self.state.stack.pop()?;

            self.state.stack.push(a + b);
        }

        Some(())
    }

    fn subtract(&mut self) -> Option<()> {
        trace!("action: subtract");

        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop()?;
            let b = self.state.stack.pop()?;

            self.state.stack.push(a - b);
        }

        Some(())
    }

    fn multiply(&mut self) -> Option<()> {
        trace!("action: multiply");

        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop()?;
            let b = self.state.stack.pop()?;

            self.state.stack.push(a * b);
        }

        Some(())
    }

    fn divide(&mut self) -> Option<()> {
        trace!("action: divide");

        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop()?;
            let b = self.state.stack.pop()?;

            self.state.stack.push(a / b);
        }

        Some(())
    }

    fn r#mod(&mut self) -> Option<()> {
        trace!("action: mod");

        if self.state.stack.len() >= 2 {
            let ref a = self.state.stack.pop()?;
            let ref b = self.state.stack.pop()?;

            let res = (a + (b % a)) % a;

            self.state.stack.push(res);
        }

        Some(())
    }

    fn not(&mut self) -> Option<()> {
        trace!("action: not");

        let val = self.state.stack.pop()?;

        if val.is_zero() {
            self.state.stack.push(1);
        } else {
            self.state.stack.push(0);
        }

        Some(())
    }

    fn greater(&mut self) -> Option<()> {
        trace!("action: greater");

        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop()?;
            let b = self.state.stack.pop()?;

            if b > a {
                self.state.stack.push(1);
            } else {
                self.state.stack.push(0);
            }
        }

        Some(())
    }

    fn pointer(&mut self) -> Option<()> {
        trace!("action: pointer");

        let n = self.state.stack.pop()?;

        let turns: BigInt = (4 + (n % 4)) % 4;

        for _ in 0..turns.to_i32().unwrap() {
            self.state.dp.rotate_clockwise();
        }

        Some(())
    }

    fn switch(&mut self) -> Option<()> {
        trace!("action: switch");

        let n = self.state.stack.pop()?;

        if n.bit(0) {
            self.state.cc = self.state.cc.toggle();
        }

        Some(())
    }

    fn duplicate(&mut self) -> Option<()> {
        trace!("action: duplicate");

        let top = self.state.stack.top().map(Clone::clone)?;
        self.state.stack.push(top);

        Some(())
    }

    #[allow(dead_code)]
    fn roll(&mut self) -> Option<()> {
        trace!("action: roll");

        unimplemented!();
    }

    #[allow(dead_code)]
    fn r#in(&mut self, iotype: IoType) -> Option<()> {
        trace!("action: in({:?})", iotype);

        // show a prompt and flush stdout
        {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            stdout.flush().expect("Failed to flush stdout.");
            write!(stdout, "? ").expect("Failed to write to stdout.");
            stdout.flush().expect("Failed to flush stdout.");
        }

        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();

        match iotype {
            IoType::Char => {
                let c = line.chars().next()?;
                self.state.stack.push(c as u32);
            }
            _ => (),
        }

        Some(())
    }

    fn out(&mut self, iotype: IoType) -> Option<()> {
        trace!("action: out({:?})", iotype);

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        match iotype {
            IoType::Char => {
                let c = self.state.stack
                    .pop()
                    .as_ref()
                    .map(ToPrimitive::to_u32)
                    .flatten()
                    .map(char::from_u32)
                    .flatten()?;

                // treat failing to write to stdout as a runtime error
                write!(handle, "{}", c).expect("Failed to write to stdout");
            }
            IoType::Number => {
                let n = self.state.stack.pop()?;

                write!(handle, "{}", n).expect("Failed to write to stdout");
            }
        }

        handle.flush().expect("Failed to flush stdout");

        Some(())
    }
}
