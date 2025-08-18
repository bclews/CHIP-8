//! Stack implementation for the Chip-8 emulator.
//!
//! This module implements the 16-level call stack used for subroutine calls
//! and returns in the Chip-8 system.

use crate::error::{EmulatorError, Result};

/// Maximum stack depth (16 levels).
pub const STACK_SIZE: usize = 16;

/// Chip-8 call stack.
/// 
/// The stack is used to store return addresses when calling subroutines.
/// It supports up to 16 levels of nested subroutine calls.
pub struct Stack {
    /// Stack data storage.
    data: [u16; STACK_SIZE],
    
    /// Current stack pointer (points to the next available position).
    sp: usize,
}

impl Stack {
    /// Creates a new empty stack.
    pub fn new() -> Self {
        Self {
            data: [0; STACK_SIZE],
            sp: 0,
        }
    }

    /// Resets the stack to empty state.
    pub fn reset(&mut self) {
        self.data.fill(0);
        self.sp = 0;
    }

    /// Pushes a value onto the stack.
    /// 
    /// # Arguments
    /// * `value` - The 16-bit value to push (typically a return address)
    /// 
    /// # Returns
    /// Ok(()) on success, or StackOverflow error if the stack is full.
    pub fn push(&mut self, value: u16) -> Result<()> {
        if self.sp >= STACK_SIZE {
            return Err(EmulatorError::StackOverflow);
        }
        
        self.data[self.sp] = value;
        self.sp += 1;
        
        Ok(())
    }

    /// Pops a value from the stack.
    /// 
    /// # Returns
    /// The popped value on success, or StackUnderflow error if the stack is empty.
    pub fn pop(&mut self) -> Result<u16> {
        if self.sp == 0 {
            return Err(EmulatorError::StackUnderflow);
        }
        
        self.sp -= 1;
        let value = self.data[self.sp];
        self.data[self.sp] = 0; // Clear the popped value for debugging
        
        Ok(value)
    }

    /// Peeks at the top value without removing it.
    /// 
    /// # Returns
    /// The top value on success, or StackUnderflow error if the stack is empty.
    pub fn peek(&self) -> Result<u16> {
        if self.sp == 0 {
            return Err(EmulatorError::StackUnderflow);
        }
        
        Ok(self.data[self.sp - 1])
    }

    /// Gets the current stack depth (number of items on the stack).
    pub fn depth(&self) -> usize {
        self.sp
    }

    /// Checks if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.sp == 0
    }

    /// Checks if the stack is full.
    pub fn is_full(&self) -> bool {
        self.sp >= STACK_SIZE
    }

    /// Gets the remaining capacity of the stack.
    pub fn remaining_capacity(&self) -> usize {
        STACK_SIZE - self.sp
    }

    /// Gets a copy of the current stack contents for debugging.
    /// 
    /// # Returns
    /// A vector containing the active stack entries (bottom to top).
    pub fn get_contents(&self) -> Vec<u16> {
        self.data[..self.sp].to_vec()
    }

    /// Clears the stack and fills it with the provided values.
    /// 
    /// This is primarily useful for testing and debugging.
    /// 
    /// # Arguments
    /// * `values` - Values to load onto the stack (bottom to top order)
    /// 
    /// # Returns
    /// Ok(()) on success, or StackOverflow if too many values are provided.
    pub fn load_values(&mut self, values: &[u16]) -> Result<()> {
        if values.len() > STACK_SIZE {
            return Err(EmulatorError::StackOverflow);
        }
        
        self.reset();
        
        for &value in values {
            self.push(value)?;
        }
        
        Ok(())
    }

    /// Gets the maximum depth the stack has reached since creation or last reset.
    /// 
    /// This is useful for profiling and debugging to understand stack usage patterns.
    pub fn max_depth_reached(&self) -> usize {
        // For now, just return current depth. In a more sophisticated implementation,
        // we could track the historical maximum.
        self.sp
    }

    /// Dumps the current stack state as a formatted string for debugging.
    pub fn dump(&self) -> String {
        if self.is_empty() {
            return "Stack: [empty]".to_string();
        }
        
        let mut result = format!("Stack (depth {}): ", self.sp);
        
        for (i, &value) in self.data[..self.sp].iter().enumerate() {
            if i > 0 {
                result.push_str(" -> ");
            }
            result.push_str(&format!("{:#04x}", value));
        }
        
        result.push_str(" (top)");
        result
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator implementation for the stack.
/// 
/// Iterates from bottom to top (oldest to newest entries).
impl<'a> IntoIterator for &'a Stack {
    type Item = &'a u16;
    type IntoIter = std::slice::Iter<'a, u16>;

    fn into_iter(self) -> Self::IntoIter {
        self.data[..self.sp].iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_creation() {
        let stack = Stack::new();
        
        assert!(stack.is_empty());
        assert!(!stack.is_full());
        assert_eq!(stack.depth(), 0);
        assert_eq!(stack.remaining_capacity(), STACK_SIZE);
    }

    #[test]
    fn test_push_pop() {
        let mut stack = Stack::new();
        
        // Test push
        stack.push(0x1234).unwrap();
        assert_eq!(stack.depth(), 1);
        assert!(!stack.is_empty());
        
        stack.push(0x5678).unwrap();
        assert_eq!(stack.depth(), 2);
        
        // Test peek
        assert_eq!(stack.peek().unwrap(), 0x5678);
        assert_eq!(stack.depth(), 2); // Peek shouldn't change depth
        
        // Test pop
        assert_eq!(stack.pop().unwrap(), 0x5678);
        assert_eq!(stack.depth(), 1);
        
        assert_eq!(stack.pop().unwrap(), 0x1234);
        assert_eq!(stack.depth(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_stack_overflow() {
        let mut stack = Stack::new();
        
        // Fill the stack to capacity
        for i in 0..STACK_SIZE {
            stack.push(i as u16).unwrap();
        }
        
        assert!(stack.is_full());
        assert_eq!(stack.remaining_capacity(), 0);
        
        // Try to push one more - should fail
        let result = stack.push(0xFFFF);
        assert!(matches!(result, Err(EmulatorError::StackOverflow)));
    }

    #[test]
    fn test_stack_underflow() {
        let mut stack = Stack::new();
        
        // Try to pop from empty stack
        let result = stack.pop();
        assert!(matches!(result, Err(EmulatorError::StackUnderflow)));
        
        // Try to peek at empty stack
        let result = stack.peek();
        assert!(matches!(result, Err(EmulatorError::StackUnderflow)));
    }

    #[test]
    fn test_stack_reset() {
        let mut stack = Stack::new();
        
        // Add some data
        stack.push(0x1111).unwrap();
        stack.push(0x2222).unwrap();
        stack.push(0x3333).unwrap();
        
        assert_eq!(stack.depth(), 3);
        
        // Reset
        stack.reset();
        
        assert!(stack.is_empty());
        assert_eq!(stack.depth(), 0);
        assert_eq!(stack.remaining_capacity(), STACK_SIZE);
    }

    #[test]
    fn test_stack_contents() {
        let mut stack = Stack::new();
        
        let values = [0x100, 0x200, 0x300, 0x400];
        
        for &value in &values {
            stack.push(value).unwrap();
        }
        
        let contents = stack.get_contents();
        assert_eq!(contents, values);
    }

    #[test]
    fn test_load_values() {
        let mut stack = Stack::new();
        
        let values = [0xABC, 0xDEF, 0x123, 0x456];
        stack.load_values(&values).unwrap();
        
        assert_eq!(stack.depth(), 4);
        assert_eq!(stack.peek().unwrap(), 0x456); // Last value should be on top
        
        // Test that values are in correct order
        assert_eq!(stack.pop().unwrap(), 0x456);
        assert_eq!(stack.pop().unwrap(), 0x123);
        assert_eq!(stack.pop().unwrap(), 0xDEF);
        assert_eq!(stack.pop().unwrap(), 0xABC);
    }

    #[test]
    fn test_load_values_overflow() {
        let mut stack = Stack::new();
        
        // Try to load more values than stack capacity
        let too_many_values: Vec<u16> = (0..STACK_SIZE + 1).map(|i| i as u16).collect();
        let result = stack.load_values(&too_many_values);
        
        assert!(matches!(result, Err(EmulatorError::StackOverflow)));
    }

    #[test]
    fn test_stack_dump() {
        let mut stack = Stack::new();
        
        // Test empty stack dump
        let dump = stack.dump();
        assert_eq!(dump, "Stack: [empty]");
        
        // Test non-empty stack dump
        stack.push(0x200).unwrap();
        stack.push(0x300).unwrap();
        stack.push(0x400).unwrap();
        
        let dump = stack.dump();
        assert!(dump.contains("Stack (depth 3)"));
        assert!(dump.contains("0x200"));
        assert!(dump.contains("0x300"));
        assert!(dump.contains("0x400"));
        assert!(dump.contains("(top)"));
    }

    #[test]
    fn test_iterator() {
        let mut stack = Stack::new();
        
        let values = [0x100, 0x200, 0x300];
        stack.load_values(&values).unwrap();
        
        let collected: Vec<u16> = stack.into_iter().copied().collect();
        assert_eq!(collected, values);
    }

    #[test]
    fn test_lifo_behavior() {
        let mut stack = Stack::new();
        
        // Push values in order
        for i in 1..=5 {
            stack.push(i * 100).unwrap();
        }
        
        // Pop values - should come out in reverse order (LIFO)
        for i in (1..=5).rev() {
            assert_eq!(stack.pop().unwrap(), i * 100);
        }
        
        assert!(stack.is_empty());
    }

    #[test]
    fn test_subroutine_simulation() {
        let mut stack = Stack::new();
        
        // Simulate nested subroutine calls
        let main_addr = 0x200;
        let sub1_addr = 0x300;
        let _sub2_addr = 0x400;
        
        // Call subroutine 1 from main
        stack.push(main_addr + 2).unwrap(); // Return address after call
        
        // Call subroutine 2 from subroutine 1
        stack.push(sub1_addr + 2).unwrap();
        
        assert_eq!(stack.depth(), 2);
        
        // Return from subroutine 2 to subroutine 1
        let return_addr = stack.pop().unwrap();
        assert_eq!(return_addr, sub1_addr + 2);
        
        // Return from subroutine 1 to main
        let return_addr = stack.pop().unwrap();
        assert_eq!(return_addr, main_addr + 2);
        
        assert!(stack.is_empty());
    }

    #[test]
    fn test_max_depth_tracking() {
        let mut stack = Stack::new();
        
        // For now, max_depth_reached just returns current depth
        // In future implementations, this could track historical maximum
        assert_eq!(stack.max_depth_reached(), 0);
        
        stack.push(0x100).unwrap();
        assert_eq!(stack.max_depth_reached(), 1);
        
        stack.push(0x200).unwrap();
        stack.push(0x300).unwrap();
        assert_eq!(stack.max_depth_reached(), 3);
        
        stack.pop().unwrap();
        assert_eq!(stack.max_depth_reached(), 2);
    }
}