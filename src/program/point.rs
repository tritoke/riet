use super::{DirectionPointer as DP, Program};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Point(pub u32, pub u32);

impl Point {
    pub fn row(&self) -> &u32 {
        &self.0
    }

    pub fn row_mut(&mut self) -> &mut u32 {
        &mut self.0
    }

    pub fn col(&self) -> &u32 {
        &self.1
    }

    pub fn col_mut(&mut self) -> &mut u32 {
        &mut self.1
    }

    pub fn next_in_direction(&self, dp: DP, program: &Program) -> Option<Self> {
        match dp {
            DP::Down if self.0 + 1 < *program.rows() => Some(Self(self.0 + 1, self.1)),
            DP::Up if self.0.checked_sub(1).is_some() => Some(Self(self.0 - 1, self.1)),
            DP::Right if self.1 + 1 < *program.cols() => Some(Self(self.0, self.1 + 1)),
            DP::Left if self.1.checked_sub(1).is_some() => Some(Self(self.0, self.1 - 1)),

            _ => None,
        }
    }
}