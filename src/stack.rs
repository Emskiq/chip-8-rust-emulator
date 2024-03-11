// Fixed size stack structure

use core::fmt;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Stack<const COUNT: usize> {
    data: [u16; COUNT],
    top: i8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StackError(pub &'static str);
impl Error for StackError { }

impl fmt::Display for StackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error while using program stack: {} ", self.0)
    }
}

impl<const COUNT: usize> Stack<COUNT> {

    pub fn new() -> Self {
        Stack{data: [0; COUNT], top: -1}
    }

    pub fn top(&self) -> Option<u16> {
        if self.top == -1 {
            None
        }
        else {
            Some(self.data[self.top as usize])
        }
    }

    pub fn push(&mut self, value: u16) -> Result<(), StackError> {
        if self.top >= 12 {
            Err(StackError("Max size of stack reached!"))
        }
        else {
            self.top += 1;
            self.data[self.top as usize] = value;
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Result<(), StackError> {
        if self.top == -1 {
            Err(StackError("Stack is empty!"))
        }
        else {
            self.data[self.top as usize] = 0;
            self.top -= 1;
            Ok(())
        }
    }
}
