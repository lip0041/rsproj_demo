use std::collections::LinkedList;
#[allow(dead_code)]
#[allow(unused)]
#[derive(Debug)]
pub struct Queue<T> {
    elements: LinkedList<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            elements: LinkedList::new(),
        }
    }

    #[allow(dead_code)]
    pub fn enqueue(&mut self, value: T) {
        self.elements.push_back(value)
    }

    #[allow(dead_code)]
    pub fn dequeue(&mut self) -> Option<T> {
        self.elements.pop_front()
    }

    #[allow(dead_code)]
    pub fn peek_front(&self) -> Option<&T> {
        self.elements.front()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Queue::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Queue;

    #[test]
    fn test_enqueue() {
        let mut queue: Queue<u8> = Queue::new();
        queue.enqueue(64);
        assert!(!queue.is_empty());
    }
}
