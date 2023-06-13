use std::cmp::Ord;
use std::default::Default;
#[derive(Debug)]
pub struct Heap<T>
where
    T: Default,
{
    count: usize,
    items: Vec<T>,
    comparator: fn(&T, &T) -> bool,
}

impl<T> Heap<T>
where
    T: Default,
{
    #[allow(dead_code)]
    pub fn new(comparator: fn(&T, &T) -> bool) -> Self {
        Self {
            count: 0,
            items: vec![T::default()],
            comparator,
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[allow(dead_code)]
    pub fn add(&mut self, value: T) {
        self.count += 1;
        self.items.push(value);

        let mut idx = self.count;
        while self.parent_idx(idx) > 0 {
            let pdx = self.parent_idx(idx);
            if (self.comparator)(&self.items[idx], &self.items[pdx]) {
                self.items.swap(idx, pdx);
            }
            idx = pdx;
        }
    }

    fn parent_idx(&self, idx: usize) -> usize {
        idx / 2
    }

    fn children_present(&self, idx: usize) -> bool {
        self.left_child_idx(idx) <= self.count
    }

    fn left_child_idx(&self, idx: usize) -> usize {
        idx * 2
    }

    fn right_child_idx(&self, idx: usize) -> usize {
        self.left_child_idx(idx) + 1
    }

    fn smallest_child_idx(&self, idx: usize) -> usize {
        if self.right_child_idx(idx) > self.count {
            self.left_child_idx(idx)
        } else {
            let ldx = self.left_child_idx(idx);
            let rdx = self.right_child_idx(idx);
            if (self.comparator)(&self.items[idx], &self.items[rdx]) {
                ldx
            } else {
                rdx
            }
        }
    }
}

impl<T> Heap<T>
where
    T: Default + Ord,
{
    #[allow(dead_code)]
    pub fn new_min() -> Heap<T> {
        Self::new(|a, b| a < b)
    }

    #[allow(dead_code)]
    pub fn new_max() -> Heap<T> {
        Self::new(|a, b| a > b)
    }
}

impl<T> Iterator for Heap<T>
where
    T: Default,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            let next = Some(self.items.swap_remove(1));
            self.count -= 1;

            if self.count > 0 {
                let mut idx = 1;
                while self.children_present(idx) {
                    let cdx = self.smallest_child_idx(idx);
                    if !(self.comparator)(&self.items[idx], &self.items[cdx]) {
                        self.items.swap(idx, cdx);
                    }
                    idx = cdx;
                }
            }
            next
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_heap() {
        let mut heap = Heap::new_min();
        heap.add(4);
        heap.add(2);
        heap.add(9);
        heap.add(11);
        println!("{:?}", heap);
        assert_eq!(heap.len(), 4);
        heap.next();
        println!("{:?}", heap);
        heap.next();
        println!("{:?}", heap);
        heap.next();
        println!("{:?}", heap);
        heap.add(1);
        println!("{:?}", heap);
    }
    #[test]
    fn test_max_heap() {
        let mut heap = Heap::new_max();
        heap.add(4);
        heap.add(2);
        heap.add(9);
        heap.add(11);
        println!("{:?}", heap);
        assert_eq!(heap.len(), 4);
        heap.next();
        println!("{:?}", heap);
        heap.next();
        println!("{:?}", heap);
        heap.next();
        println!("{:?}", heap);
        heap.add(1);
        println!("{:?}", heap);
    }

    #[derive(Default, Debug)]
    struct Point(i32, i32);

    #[test]
    fn test_key_heap() {
        let mut heap: Heap<Point> = Heap::new(|a, b| a.0 < b.0);
        heap.add(Point(1, 15));
        heap.add(Point(3, 10));
        heap.add(Point(-2, 3));
        println!("{:?}", heap);
        heap.next();
        println!("{:?}", heap);
    }
}
