use super::stack::Stack;

#[derive(Debug)]
pub struct QueueUsingStack<T> {
    s1: Stack<T>,
    s2: Stack<T>,
}

impl<T> QueueUsingStack<T>
where
    T: Copy,
{
    #[allow(dead_code)]
    pub fn new() -> QueueUsingStack<T> {
        QueueUsingStack {
            s1: Stack::new(),
            s2: Stack::new(),
        }
    }

    #[allow(dead_code)]
    pub fn enqueue(&mut self, value: T) {
        self.s1.push(value);
    }

    #[allow(dead_code)]
    pub fn dequeue(&mut self) -> Option<T> {
        if self.s2.is_empty() {
            while !self.s1.is_empty() {
                self.s2.push(self.s1.pop().unwrap())
            }
        }
        self.s2.pop()
    }

    #[allow(dead_code)]
    pub fn peek_front(&mut self) -> Option<&T> {
        if self.s2.is_empty() {
            while !self.s1.is_empty() {
                self.s2.push(self.s1.pop().unwrap())
            }
        }
        self.s2.top()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.s1.len() + self.s2.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.s1.is_empty() && self.s2.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::QueueUsingStack;

    #[test]
    fn test_enqueue() {
        let mut queue: QueueUsingStack<u8> = QueueUsingStack::new();
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        queue.enqueue(4);
        println!("{:?}", queue);
        queue.dequeue();
        println!("{:?}", queue);
        assert_eq!(queue.peek_front(), Some(&2));
    }
}
