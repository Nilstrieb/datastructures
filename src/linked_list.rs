use std::fmt::Debug;
use std::ptr::NonNull;

type NodePtr<T> = Option<NonNull<Node<T>>>;

/// A doubly linked list with unsafe :O, except it's kind of shit compared to the std one
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

#[derive(Debug)]
pub struct Node<T> {
    value: T,
    next: NodePtr<T>,
    prev: NodePtr<T>,
}

impl<T> LinkedList<T> {
    /// Creates a new empty Linked List
    pub fn new() -> LinkedList<T> {
        Self {
            start: None,
            end: None,
        }
    }

    /// Prepend an element to the start of the list
    pub fn prepend(&mut self, element: T) {
        match self.start {
            None => {
                let node = allocate_nonnull(Node {
                    value: element,
                    next: None,
                    prev: None,
                });
                self.start = Some(node);
                self.end = Some(node);
            }
            Some(mut node) => {
                let new = allocate_nonnull(Node {
                    value: element,
                    next: Some(node),
                    prev: None,
                });
                unsafe { node.as_mut() }.prev = Some(new);
            }
        }
    }

    /// Random access over the list
    pub fn get(&self, index: usize) -> Option<&T> {
        dbg!(self);
        Self::get_inner(self.start.as_ref(), index).map(|n| &n.value)
    }

    fn get_inner(node: Option<&NonNull<Node<T>>>, index: usize) -> Option<&Node<T>> {
        if let Some(ptr) = node {
            match index {
                0 => unsafe { Some(ptr.as_ref()) },
                n => LinkedList::get_inner(Some(ptr), n - 1),
            }
        } else {
            None
        }
    }
}

fn allocate_nonnull<T>(element: T) -> NonNull<T> {
    let mut boxed = Box::new(element);
    NonNull::new(boxed.as_mut()).expect("Allocation returned null pointer")
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

pub struct IntoIter<T> {
    item: NodePtr<T>,
}

impl<T> IntoIter<T> {
    fn new(list: LinkedList<T>) -> Self {
        Self { item: list.start }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        //let next = self.item.take();
        //let ptr = match next {
        //    None => return None,
        //    Some(mut ptr) => unsafe { ptr.as_mut() },
        //};

        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn random_access() {
        let mut list = LinkedList::new();
        list.prepend("hallo");
        list.prepend("test");
        list.prepend("nice");
        assert_eq!(list.get(0), Some(&"nice"));
        assert_eq!(list.get(1), Some(&"test"));
        assert_eq!(list.get(2), Some(&"hallo"));
        assert_eq!(list.get(3), None);
    }
}
