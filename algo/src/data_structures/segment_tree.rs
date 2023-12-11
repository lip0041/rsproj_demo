#![allow(unused)]
use std::cmp::min;
use std::fmt::Debug;
use std::ops::Range;

pub struct SegmentTree<T: Debug + Default + Ord + Copy> {
    len: usize,
    tree: Vec<T>,
    merge: fn(T, T) -> T,
}

impl<T: Debug + Default + Ord + Copy> SegmentTree<T> {
    pub fn from_vec(arr: &[T], merge: fn(T, T) -> T) -> Self {
        let len = arr.len();
        let mut buf: Vec<T> = vec![T::default(); 2 * len];

        buf[len..(2 * len)].clone_from_slice(&arr[0..len]);
        for i in (1..len).rev() {
            buf[i] = merge(buf[2 * i], buf[2 * i + 1]);
        }

        SegmentTree {
            len,
            tree: buf,
            merge,
        }
    }

    pub fn query(&self, range: Range<usize>) -> Option<T> {
        let mut l = range.start + self.len;
        let mut r = min(self.len, range.end) + self.len;
        let mut res = None;

        while l < r {
            if l % 2 == 1 {
                res = Some(match res {
                    None => self.tree[l],
                    Some(old) => (self.merge)(old, self.tree[l]),
                });
                l += 1;
            }
            if r % 2 == 1 {
                r -= 1;
                res = Some(match res {
                    None => self.tree[r],
                    Some(old) => (self.merge)(old, self.tree[r]),
                });
            }
            l /= 2;
            r /= 2;
        }
        res
    }

    pub fn update(&mut self, idx: usize, val: T) {
        let mut idx = idx + self.len;
        self.tree[idx] = val;

        idx /= 2;
        while idx != 0 {
            self.tree[idx] = (self.merge)(self.tree[2 * idx], self.tree[2 * idx + 1]);
            idx /= 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::{max, min};

    #[test]
    fn test_min_segments() {
        let vec = vec![-30, 2, -4, 7, 3, -5, 6, 11, -20, 9, 14, 15, 5, 2, -8];
        let min_seg_tree = SegmentTree::from_vec(&vec, min);
        assert_eq!(Some(-5), min_seg_tree.query(4..7));
        assert_eq!(Some(-30), min_seg_tree.query(0..vec.len()));
        assert_eq!(Some(-30), min_seg_tree.query(0..2));
        assert_eq!(Some(-4), min_seg_tree.query(1..3));
        assert_eq!(Some(-5), min_seg_tree.query(1..7));
    }

    #[test]
    fn test_max_segments() {
        let val_at_6 = 6;
        let vec = vec![1, 2, -4, 7, 3, -5, val_at_6, 11, -20, 9, 14, 15, 5, 2, -8];
        let mut max_seg_tree = SegmentTree::from_vec(&vec, max);
        assert_eq!(Some(15), max_seg_tree.query(0..vec.len()));
        let max_4_to_6 = 6;
        assert_eq!(Some(max_4_to_6), max_seg_tree.query(4..7));
        let delta = 2;
        max_seg_tree.update(6, val_at_6 + delta);
        assert_eq!(Some(val_at_6 + delta), max_seg_tree.query(4..7));
    }

    #[test]
    fn test_sum_segments() {
        let val_at_6 = 6;
        let vec = vec![1, 2, -4, 7, 3, -5, val_at_6, 11, -20, 9, 14, 15, 5, 2, -8];
        let mut sum_seg_tree = SegmentTree::from_vec(&vec, |a, b| a + b);
        for (i, val) in vec.iter().enumerate() {
            assert_eq!(Some(*val), sum_seg_tree.query(i..(i + 1)));
        }
        let sum_4_to_6 = sum_seg_tree.query(4..7);
        assert_eq!(Some(4), sum_4_to_6);
        let delta = 3;
        sum_seg_tree.update(6, val_at_6 + delta);
        assert_eq!(
            sum_4_to_6.unwrap() + delta,
            sum_seg_tree.query(4..7).unwrap()
        );
    }

}
