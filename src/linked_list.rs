pub struct LinkedList {}

pub struct Node<T> {
    value: T,
    next: *const Node<T>,
    prev: *const Node<T>,
}


