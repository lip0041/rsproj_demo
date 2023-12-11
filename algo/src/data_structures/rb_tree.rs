#![allow(unused)]
use std::boxed::Box;
use std::cmp::{Ord, Ordering};
use std::iter::Iterator;
use std::ptr::null_mut;

#[derive(Copy, Clone)]
enum Color {
    Red,
    Black,
}

pub struct RBNode<K: Ord, V> {
    key: K,
    value: V,
    color: Color,
    parent: *mut RBNode<K, V>,
    left: *mut RBNode<K, V>,
    right: *mut RBNode<K, V>,
}

impl<K: Ord, V> RBNode<K, V> {
    fn new(key: K, value: V) -> RBNode<K, V> {
        RBNode {
            key,
            value,
            color: Color::Red,
            parent: null_mut(),
            left: null_mut(),
            right: null_mut(),
        }
    }
}

pub struct RBTree<K: Ord, V> {
    root: *mut RBNode<K, V>,
}

impl<K: Ord, V> Default for RBTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> RBTree<K, V> {
    pub fn new() -> RBTree<K, V> {
        RBTree { root: null_mut() }
    }

    pub fn find(&self, key: &K) -> Option<&V> {
        unsafe {
            let mut node = self.root;
            while !node.is_null() {
                node = match (*node).key.cmp(key) {
                    Ordering::Less => (*node).right,
                    Ordering::Equal => return Some(&(*node).value),
                    Ordering::Greater => (*node).left,
                }
            }
        }
        None
    }

    pub fn insert(&mut self, key: K, value: V) {
        unsafe {
            let mut parent = null_mut();
            let mut node = self.root;
            while !node.is_null() {
                parent = node;
                node = match (*node).key.cmp(&key) {
                    Ordering::Less => (*node).right,
                    Ordering::Equal => {
                        (*node).value = value;
                        return;
                    }
                    Ordering::Greater => (*node).left,
                }
            }

            node = Box::into_raw(Box::new(RBNode::new(key, value)));

            if !parent.is_null() {
                if (*node).key < (*parent).key {
                    (*parent).left = node;
                } else {
                    (*parent).right = node;
                }
            } else {
                self.root = node;
            }

            (*node).parent = parent;
            insert_fixup(self, node);
        }
    }

    pub fn delete(&mut self, key: &K) {
        unsafe {
            let mut parent = null_mut();
            let mut node = self.root;
            while !node.is_null() {
                node = match (*node).key.cmp(key) {
                    Ordering::Less => {
                        parent = node;
                        (*node).right
                    }
                    Ordering::Equal => break,
                    Ordering::Greater => {
                        parent = node;
                        (*node).left
                    }
                };
            }

            if node.is_null() {
                return;
            }

            let cl = (*node).left;
            let cr = (*node).right;
            let mut deleted_color;

            if cl.is_null() {
                replace_node(self, parent, node, cr);
                if cr.is_null() {
                    deleted_color = (*node).color;
                } else {
                    (*cr).parent = parent;
                    (*cr).color = Color::Black;
                    deleted_color = Color::Red;
                }
            } else if cr.is_null() {
                replace_node(self, parent, node, cl);
                (*cl).parent = parent;
                (*cl).color = Color::Black;
                deleted_color = Color::Red;
            } else {
                let mut victim = (*node).right;
                while !(*victim).left.is_null() {
                    victim = (*victim).left;
                }

                if victim == (*node).right {
                    replace_node(self, parent, node, victim);
                    (*victim).parent = parent;
                    deleted_color = (*victim).color;
                    (*victim).color = (*node).color;
                    (*victim).left = cl;
                    (*cl).parent = victim;
                    if (*victim).right.is_null() {
                        parent = victim;
                    } else {
                        deleted_color = Color::Red;
                        (*(*victim).right).color = Color::Black;
                    }
                }
            }
        }
    }

    pub fn iter<'a>(&self) -> RBTreeIterator<'a, K, V> {
        let mut iterator = RBTreeIterator { stack: Vec::new() };
        let mut node = self.root;
        unsafe {
            while !node.is_null() {
                iterator.stack.push(&*node);
                node = (*node).left;
            }
        }
        iterator
    }
}

pub struct RBTreeIterator<'a, K: Ord, V> {
    stack: Vec<&'a RBNode<K, V>>,
}

impl<'a, K: Ord, V> Iterator for RBTreeIterator<'a, K, V> {
    type Item = &'a RBNode<K, V>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.stack.pop() {
            Some(node) => {
                let mut next = node.right;
                unsafe {
                    while !next.is_null() {
                        self.stack.push(&*next);
                        next = (*next).left;
                    }
                }
                Some(node)
            }
            None => None,
        }
    }
}

#[inline]
unsafe fn insert_fixup<K: Ord, V>(tree: &mut RBTree<K, V>, mut node: *mut RBNode<K, V>) {
    let mut parent = (*node).parent;
    let mut gparent: *mut RBNode<K, V>;
    let mut uncle: *mut RBNode<K, V>;

    loop {
        if parent.is_null() {
            (*node).color = Color::Black;
            break;
        }

        if matches!((*parent).color, Color::Black) {
            break;
        }

        gparent = (*parent).parent;
        // assume uncle is the right
        uncle = (*gparent).right;

        if parent != uncle {
            // parent = (*gparent).left
            if !uncle.is_null() && matches!((*uncle).color, Color::Red) {
                // case 1: uncle is right red
                // if node is null, it is black
                (*parent).color = Color::Black;
                (*uncle).color = Color::Black;
                (*gparent).color = Color::Red;
                node = gparent;
                parent = (*node).parent;
                continue;
            }

            if node == (*parent).right {
                // case 2: uncle is black
                // black left right
                left_rotate(tree, parent);
                parent = node;
            }

            // case 3: black left left
            (*parent).color = Color::Black;
            (*gparent).color = Color::Red;
            right_rotate(tree, gparent);
        } else {
            // parent = (*gparent).right
            uncle = (*gparent).left;
            if !uncle.is_null() && matches!((*uncle).color, Color::Red) {
                // case 1: uncle is right red
                (*parent).color = Color::Black;
                (*uncle).color = Color::Black;
                (*gparent).color = Color::Red;
                node = gparent;
                parent = (*node).parent;
                continue;
            }

            if node == (*parent).left {
                // case 3: black right left
                right_rotate(tree, parent);
                parent = node;
            }

            // case 4: black right right
            (*parent).color = Color::Black;
            (*gparent).color = Color::Red;
            left_rotate(tree, gparent);
        }
        break;
    }
}

#[inline]
unsafe fn left_rotate<K: Ord, V>(tree: &mut RBTree<K, V>, x: *mut RBNode<K, V>) {
    // left rotate at x
    // x will be downgraded
    // x's right wiil be upgraded
    let p = (*x).parent;
    let y = (*x).right;
    let c = (*y).left;

    (*y).left = x;
    (*x).parent = y;
    (*x).right = c;
    if !c.is_null() {
        (*c).parent = x;
    }

    if p.is_null() {
        tree.root = y;
    } else if (*p).left == x {
        (*p).left = y;
    } else {
        (*p).right = y;
    }
    (*y).parent = p;
}

#[inline]
unsafe fn right_rotate<K: Ord, V>(tree: &mut RBTree<K, V>, x: *mut RBNode<K, V>) {
    // right rotate at x
    // x will be downgraded
    // x's left wiil be upgraded
    let p = (*x).parent;
    let y = (*x).left;
    let c = (*y).right;

    (*y).right = x;
    (*x).parent = y;
    (*x).left = c;
    if !c.is_null() {
        (*c).parent = x;
    }

    if p.is_null() {
        tree.root = y;
    } else if (*p).left == x {
        (*p).left = y;
    } else {
        (*p).right = y;
    }
    (*y).parent = p;
}

#[inline]
unsafe fn replace_node<K: Ord, V>(
    tree: &mut RBTree<K, V>,
    parent: *mut RBNode<K, V>,
    node: *mut RBNode<K, V>,
    new: *mut RBNode<K, V>,
) {
    if parent.is_null() {
        tree.root = new;
    } else if (*parent).left == node {
        (*parent).left = new;
    } else {
        (*parent).right = new;
    }
}

#[cfg(test)]
mod tests {
    use super::RBTree;

    #[test]
    fn find() {
        let mut tree = RBTree::<usize, char>::new();
        for (k, v) in "hello, world!".chars().enumerate() {
            tree.insert(k, v);
        }
        assert_eq!(*tree.find(&3).unwrap_or(&'*'), 'l');
        assert_eq!(*tree.find(&6).unwrap_or(&'*'), ' ');
        assert_eq!(*tree.find(&8).unwrap_or(&'*'), 'o');
        assert_eq!(*tree.find(&12).unwrap_or(&'*'), '!');
    }

    #[test]
    fn insert() {
        let mut tree = RBTree::<usize, char>::new();
        for (k, v) in "hello, world!".chars().enumerate() {
            tree.insert(k, v);
        }
        let s: String = tree.iter().map(|x| x.value).collect();
        assert_eq!(s, "hello, world!");
    }

    #[test]
    fn delete() {
        let mut tree = RBTree::<usize, char>::new();
        for (k, v) in "hello, world!".chars().enumerate() {
            tree.insert(k, v);
        }
        tree.delete(&1);
        tree.delete(&3);
        tree.delete(&5);
        tree.delete(&7);
        tree.delete(&11);
        let s: String = tree.iter().map(|x| x.value).collect();
        assert_eq!(s, "hlo orl!");
    }
}
