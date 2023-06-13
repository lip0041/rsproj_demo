use super::queue::Queue;

#[derive(Debug)]
pub struct StackUsingQueue<T> {
    q1: Queue<T>,
    q2: Queue<T>,
}

impl<T> StackUsingQueue<T>
where
    T: Clone + Copy,
{
    #[allow(dead_code)]
    pub fn new() -> Self {
        StackUsingQueue {
            q1: Queue::new(),
            q2: Queue::new(),
        }
    }

    #[allow(dead_code)]
    pub fn push(&mut self, value: T) {
        if self.q1.is_empty() && self.q2.is_empty() {
            self.q1.enqueue(value);
            return;
        }
        if self.q1.is_empty() {
            self.q2.enqueue(value)
        } else {
            self.q1.enqueue(value)
        }
    }

    #[allow(dead_code)]
    pub fn pop(&mut self) -> Option<T> {
        if self.q1.is_empty() {
            while self.q2.len() > 1 {
                self.q1.enqueue(self.q2.dequeue().unwrap());
            }
            self.q2.dequeue()
        } else {
            while self.q1.len() > 1 {
                self.q2.enqueue(self.q1.dequeue().unwrap());
            }
            self.q1.dequeue()
        }
    }

    #[allow(dead_code)]
    pub fn top(&mut self) -> Option<T> {
        let res = self.pop();
        if self.q1.is_empty() {
            self.q2.enqueue(res.unwrap().clone())
        } else {
            self.q1.enqueue(res.unwrap().clone())
        }
        res
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        if self.q1.is_empty() {
            self.q2.len()
        } else {
            self.q1.len()
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.q1.is_empty() && self.q2.is_empty()
    }
}

#[cfg(test)]
mod tests {

    use super::StackUsingQueue;

    #[test]
    fn test_push() {
        let mut stack: StackUsingQueue<i32> = StackUsingQueue::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);
        stack.push(4);
        println!("{:?}", stack);
        stack.pop();
        stack.push(5);
        stack.push(6);
        stack.pop();
        println!("{:?}", stack);
        assert_eq!(stack.top(), Some(5));
    }
}
