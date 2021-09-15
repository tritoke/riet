use crate::program::{CodelChooser, Color, DirectionPointer, Point, Program};

use std::collections::HashSet;
use std::convert::TryInto;
use std::fmt;
use std::io::{self, prelude::*};

use anyhow::{bail, ensure};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use num_traits::{One, Signed, Zero};

#[derive(Debug, Default, Clone)]
struct PietState {
    dp: DirectionPointer,
    cc: CodelChooser,
    curr_codel: Point,
    stack: Vec<BigInt>,
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

impl fmt::Display for IoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[rustfmt::skip]
        match self {
            IoType::Char   => write!(f, "char")?,
            IoType::Number => write!(f, "number")?,
        }

        Ok(())
    }
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
        let (curr, curr_color, next, next_color) = {
            let mut curr = self.state.curr_codel;
            let mut curr_color = *self.program.get_codel(*curr.row(), *curr.col()).unwrap();

            ensure!(
                curr_color != Color::Black,
                "Cannot execute from a inside black block"
            );

            let mut next;
            let mut next_color;

            if matches!(curr_color, Color::White) {
                trace!("Entering white block at {:?} {:?}|{:?}", curr, self.state.dp, self.state.cc);

                // go in a straight line until we encounter a restriction or a non-white pixel
                let mut seen_states: HashSet<(Point, DirectionPointer, CodelChooser)> =
                    Default::default();

                loop {
                    if !seen_states.insert((curr, self.state.dp, self.state.cc)) {
                        trace!("Could not escape white block - exiting");
                        std::process::exit(0);
                    }

                    let next_codel = curr.next_in_direction(self.state.dp, &self.program);
                    let maybe_next_color = next_codel
                        .and_then(|Point(row, col)| self.program.get_codel(row, col).copied());

                    // restricted
                    if next_codel.is_none() || matches!(maybe_next_color, Some(Color::Black)) {
                        self.state.cc = self.state.cc.toggle();
                        self.state.dp = self.state.dp.rotate_clockwise();
                    } else {
                        next = next_codel.unwrap();
                        next_color = maybe_next_color.unwrap();

                        if matches!(next_color, Color::White) {
                            curr = next;
                            curr_color = next_color;
                        } else {
                            trace!("white cell(s) crossed, continuing at {:?}", next);
                            
                            self.state.curr_codel = curr;

                            break (curr, curr_color, next, next_color);
                        }
                    }
                }
            } else {
                let mut next: Option<Point> = None;
                let mut next_color: Option<Color> = None;
                let mut escaped = false;
                
                for tries in 0..8 {
                    let edge = self
                        .program
                        .get_color_block(&self.state.curr_codel)
                        .map(|cb| cb.edge(self.state.dp, self.state.cc))
                        .unwrap();
                    
                    let next_codel = edge.next_in_direction(self.state.dp, &self.program);

                    if let Some(Point(row, col)) = next_codel {
                        next = next_codel;
                        next_color = self.program.get_codel(row, col).copied();

                        if !matches!(next_color, Some(Color::Black)) {
                            escaped = true;
                            break;
                        }
                    }
                    
                    if tries % 2 == 0 {
                        self.state.cc = self.state.cc.toggle();
                    } else {
                        self.state.dp = self.state.dp.rotate_clockwise();
                    }
                }

                if escaped {
                    (curr, curr_color, next.unwrap(), next_color.unwrap())
                } else {
                    trace!("Attempted to exit block 8 times, exiting.");

                    std::process::exit(0);
                }
            }
        };

        let block_value = self.program
            .get_color_block(&self.state.curr_codel)
            .map(|cb| cb.num_codels())
            .unwrap();

        trace!(
            "step {:}  {:?} {:?}|{:?} {:?} -> {:?} {:?}|{:?} {:?}",
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

        self.action(curr_color, next_color, block_value)?;

        trace!("stack: {:?}", self.state.stack);

        self.step_no += 1;
        self.state.curr_codel = next;

        Ok(())
    }

    fn action(
        &mut self,
        curr_color: Color,
        next_color: Color,
        block_value: usize,
    ) -> anyhow::Result<()> {
        let hue_change = curr_color.hue_change(&next_color);
        let lightness_change = curr_color.lightness_change(&next_color);

        if let (Some(hc), Some(lc)) = (hue_change, lightness_change) {
            // Hue change  None       1 Darker     2 Darker
            // None                   push         pop
            // 1 Step      add        subtract     multiply
            // 2 Steps     divide     mod          not
            // 3 Steps     greater    pointer      switch
            // 4 Steps     duplicate  roll         in(number)
            // 5 Steps     in(char)   out(number)  out(char)

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
                (4, 1) => { self.roll(); },
                (4, 2) => { self.r#in(IoType::Number); },

                (5, 0) => { self.r#in(IoType::Char); },
                (5, 1) => { self.out(IoType::Number); },
                (5, 2) => { self.out(IoType::Char); },

                (_, _) => { bail!("Unknown hue/lightness change: (lc:{:?}, hc:{:?})", lc, hc) }
            }
        }

        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<!> {
        loop {
            self.step()?
        }
    }

    pub fn run_until(&mut self, max_steps: usize) -> anyhow::Result<()> {
        while self.step_no < max_steps {
            self.step()?
        }

        info!(
            "Program stopping: reached maximum number of steps - {}",
            max_steps
        );

        Ok(())
    }

    fn push(&mut self, v: usize) {
        trace!("action: push, value {:?}", v);

        self.state.stack.push(v.into());
    }

    fn pop(&mut self) {
        trace!("action: pop");

        if self.state.stack.pop().is_none() {
            info!("pop: empty stack");
        }
    }

    fn add(&mut self) -> Option<()> {
        trace!("action: add");

        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop()?;
            let b = self.state.stack.pop()?;

            self.state.stack.push(a + b);
        } else {
            info!("add failed: stack underflow");
        }

        Some(())
    }

    fn subtract(&mut self) -> Option<()> {
        trace!("action: subtract");

        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop()?;
            let b = self.state.stack.pop()?;

            self.state.stack.push(b - a);
        } else {
            info!("subtract failed: stack underflow");
        }

        Some(())
    }

    fn multiply(&mut self) -> Option<()> {
        trace!("action: multiply");

        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop()?;
            let b = self.state.stack.pop()?;

            self.state.stack.push(a * b);
        } else {
            info!("multiply failed: stack underflow");
        }

        Some(())
    }

    fn divide(&mut self) -> Option<()> {
        trace!("action: divide");

        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop()?;
            let b = self.state.stack.pop()?;

            if a.is_zero() {
                info!("divide failed: division by zero");

                return None;
            }

            self.state.stack.push(b / a);
        } else {
            info!("divide failed: stack underflow");
        }

        Some(())
    }

    fn r#mod(&mut self) -> Option<()> {
        trace!("action: mod");

        if self.state.stack.len() >= 2 {
            let a = &self.state.stack.pop()?;
            let b = &self.state.stack.pop()?;

            if a.is_zero() {
                info!("mod failed: division by zero");

                return None;
            }

            let res = (a + (b % a)) % a;

            self.state.stack.push(res);
        } else {
            info!("mod failed: stack underflow");
        }

        Some(())
    }

    fn not(&mut self) -> Option<()> {
        trace!("action: not");

        let top = self.state.stack.pop();

        if top.is_none() {
            info!("not failed: stack underflow");
        }

        let val = top?;

        if val.is_zero() {
            self.state.stack.push(One::one());
        } else {
            self.state.stack.push(Zero::zero());
        }

        Some(())
    }

    fn greater(&mut self) -> Option<()> {
        trace!("action: greater");

        if self.state.stack.len() >= 2 {
            let a = self.state.stack.pop()?;
            let b = self.state.stack.pop()?;

            if b > a {
                self.state.stack.push(One::one());
            } else {
                self.state.stack.push(Zero::zero());
            }
        } else {
            info!("greater failed: stack underflow");
        }

        Some(())
    }

    fn pointer(&mut self) -> Option<()> {
        trace!("action: pointer");

        let top = self.state.stack.pop();

        if top.is_none() {
            info!("pointer failed: stack underflow");
        }

        let n = top?;

        let turns: BigInt = (4 + (n % 4)) % 4;

        for _ in 0..turns.to_u32().unwrap() {
            self.state.dp = self.state.dp.rotate_clockwise();
        }

        Some(())
    }

    fn switch(&mut self) -> Option<()> {
        trace!("action: switch");

        let top = self.state.stack.pop();

        if top.is_none() {
            info!("switch failed: stack underflow");
        }

        let n = top?;

        if n.bit(0) {
            self.state.cc = self.state.cc.toggle();
        }

        Some(())
    }

    fn duplicate(&mut self) -> Option<()> {
        trace!("action: duplicate");

        let top = self.state.stack.last().map(Clone::clone);

        if top.is_none() {
            info!("duplicate failed: stack underflow");
        }

        self.state.stack.push(top?);

        Some(())
    }

    fn roll(&mut self) -> Option<()> {
        trace!("action: roll");

        if self.state.stack.len() >= 2 {
            let rolls = self.state.stack.pop()?;
            let depth: usize = {
                let d = self.state.stack.pop()?;

                if d.is_negative() {
                    info!("roll failed: negative depth");

                    return None;
                }

                let d_us = d.try_into().ok();

                if d_us.is_none() {
                    info!("roll failed: depth exceeds maximum value of usize")
                }

                d_us?
            };

            if depth > self.state.stack.len() {
                info!("roll failed: depth exceeds stack size");

                return None;
            }

            let stack_len = self.state.stack.len();
            let section = &mut self.state.stack[stack_len - depth..];

            let mid = (rolls.magnitude() % depth).try_into().unwrap();
            if rolls.is_negative() {
                section.rotate_right(mid);
            } else {
                section.rotate_left(mid);
            }
        } else {
            info!("roll failed: stack underflow");
        }

        Some(())
    }

    fn r#in(&mut self, iotype: IoType) -> Option<()> {
        trace!("action: in({})", iotype);

        // show a prompt and flush stdout
        {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            write!(stdout, "> ").expect("Failed to write to stdout");
            stdout.flush().expect("Failed to flush stdout");
        }

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read from stdin");

        match iotype {
            IoType::Char => {
                let c = line.chars().next();

                if c.is_none() {
                    info!("in(char) failed: input contained no characters");
                }

                self.state.stack.push((c? as u32).into());
            }
            IoType::Number => {
                let num = line.trim().parse::<BigInt>().ok();

                if num.is_none() {
                    info!("in(number) failed: input was not a valid number");
                }

                self.state.stack.push(num?);
            }
        }

        Some(())
    }

    fn out(&mut self, iotype: IoType) -> Option<()> {
        trace!("action: out({})", iotype);

        let top = self.state.stack.pop();

        if top.is_none() {
            info!("out(char) failed: stack underflow")
        }

        match iotype {
            IoType::Char => {
                let c = top?.to_u32().and_then(char::from_u32);

                if c.is_none() {
                    info!("out(char) failed: value popped off the stack was not a valid char")
                }

                // treat failing to write to stdout as a runtime error
                write!(io::stdout(), "{}", c?).expect("Failed to write to stdout");
            }
            IoType::Number => {
                write!(io::stdout(), "{}", top?).expect("Failed to write to stdout");
            }
        }

        io::stdout().flush().expect("Failed to flush stdout.");

        Some(())
    }
}
