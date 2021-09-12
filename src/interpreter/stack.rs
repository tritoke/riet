use num_bigint::BigInt;

use std::collections::VecDeque;

#[derive(Debug, Default, Clone)]
pub struct Stack {
    store: VecDeque<BigInt>,
}

impl Stack {
    pub fn push(&mut self, v: impl Into<BigInt>) {
        self.store.push_back(v.into())
    }

    pub fn pop(&mut self) -> Option<BigInt> {
        self.store.pop_back()
    }

    pub fn top(&self) -> Option<&BigInt> {
        self.store.back()
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}