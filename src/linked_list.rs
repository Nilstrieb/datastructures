use std::ptr::NonNull;

#[derive(Debug)]
pub struct LinkedList<T> {
    start: Option<NonNull<Node<T>>>,
    end: Option<NonNull<Node<T>>>,
}

impl<T> Clone for LinkedList<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        todo!()
    }
}

pub struct Node<T> {
    value: T,
    next: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self<T> {
        Self {
            start: None,
            end: None,
        }
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

pub struct IntoIter<T> {
    item: Option<NonNull<Node<T>>>,
}

impl<T> IntoIter<T> {
    fn new(list: LinkedList<T>) -> Self {
        Self { item: list.start }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.item.take();
        let ptr = match next {
            None => return None,
            Some(mut ptr) => unsafe { ptr.as_mut() },
        };

        todo!()
    }
}
