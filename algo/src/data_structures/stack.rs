use std::collections::LinkedList;
#[derive(Debug)]
pub struct Stack<T> {
    elements: LinkedList<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack {
            elements: LinkedList::new(),
        }
    }

    #[allow(dead_code)]
    pub fn push(&mut self, value: T) {
        self.elements.push_back(value)
    }

    #[allow(dead_code)]
    pub fn pop(&mut self) -> Option<T> {
        self.elements.pop_back()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    #[allow(dead_code)]
    pub fn top(&self) -> Option<&T> {
        self.elements.back()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Stack::new()
    }
}

#[cfg(test)]
mod tests {

    use super::Stack;

    #[test]
    fn test_push() {
        let mut stack: Stack<i32> = Stack::new();
        stack.push(64);
        assert_eq!(stack.top(), Some(64).as_ref());
    }
}
