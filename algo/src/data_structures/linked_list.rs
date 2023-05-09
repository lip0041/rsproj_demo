struct Node<T> {
    x: T,
    y: T,
}

enum List<T> {
    Cons(Node<T>, Box<List<T>>),
    Nil,
}
use List::*;
impl<T> List<T> {
    fn new() -> List<T> {
        Nil
    }
}
