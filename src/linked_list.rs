use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A doubly linked list with unsafe :O, except it's kind of shit compared to the std one
pub struct LinkedList<T> {
    start: Option<NonNull<Node<T>>>,
    end: Option<NonNull<Node<T>>>,
    _maker: PhantomData<T>,
}

impl<T> LinkedList<T> {
    /// Creates a new empty Linked List
    pub fn new() -> LinkedList<T> {
        std::collections::LinkedList::new().push_front("hi");
        Self {
            start: None,
            end: None,
            _maker: PhantomData,
        }
    }

    /// Prepend an element to the start of the list
    pub fn push_front(&mut self, element: T) {
        match self.start {
            // empty list
            None => {
                let node = allocate_nonnull(Node {
                    value: element,
                    next: None,
                    prev: None,
                    _maker: PhantomData,
                });
                self.start = Some(node);
                self.end = Some(node);
            }
            // at lest one element
            Some(mut node) => {
                let new = allocate_nonnull(Node {
                    value: element,
                    next: Some(node),
                    prev: None,
                    _maker: PhantomData,
                });
                // SAFETY: All pointers should always be valid
                unsafe { node.as_mut() }.prev = Some(new);
                self.start = Some(new);
            }
        }
    }

    pub fn _insert_end(&mut self, _element: T) {}

    pub fn get(&self, mut index: usize) -> Option<&T> {
        let mut node = &self.start;
        let mut result = None;
        while let Some(content) = node {
            // SAFETY: All pointers should always be valid
            let content = unsafe { content.as_ref() };
            if index == 0 {
                result = Some(&content.value);
                break;
            }
            index -= 1;
            node = &content.next;
        }
        result
    }

    pub fn _get_node(&self, _index: usize) {}

    /// Returns an iterator over the items
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }
}

impl<T> Debug for LinkedList<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut item = self.start;
        while let Some(content) = item {
            // SAFETY: All pointers should always be valid and created from a box
            unsafe {
                item = content.as_ref().next;
                Box::from_raw(content.as_ptr());
            }
        }
    }
}

/// A Node in a `LinkedList`
#[derive(Debug)]
pub struct Node<T> {
    value: T,
    next: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
    _maker: PhantomData<T>,
}

fn allocate_nonnull<T>(element: T) -> NonNull<T> {
    let boxed = Box::new(element);
    // SAFETY: box is always non-null
    unsafe { NonNull::new_unchecked(Box::leak(boxed)) }
}

pub struct Iter<'a, T> {
    item: Option<&'a Node<T>>,
}

impl<'a, T> Iter<'a, T> {
    fn new(list: &'a LinkedList<T>) -> Self {
        Self {
            item: list.start.as_ref().map(|nn| {
                // SAFETY: All pointers should always be valid, the list lives as long as its items
                unsafe { nn.as_ref() }
            }),
        }
    }
}

impl<'a, T: Debug> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.item;
        match current {
            Some(node) => {
                self.item = node.next.as_ref().map(|nn| {
                    // SAFETY: All pointers should always be valid
                    unsafe { nn.as_ref() }
                });
                Some(&node.value)
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn random_access() {
        let mut list = LinkedList::new();
        list.push_front("hallo");
        list.push_front("test");
        list.push_front("nice");
        assert_eq!(list.get(0), Some(&"nice"));
        assert_eq!(list.get(1), Some(&"test"));
        assert_eq!(list.get(2), Some(&"hallo"));
        assert_eq!(list.get(3), None);
    }

    #[test]
    fn iter_simple() {
        let mut list = LinkedList::new();
        list.push_front("hallo");
        list.push_front("test");
        list.push_front("nice");
        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&"nice"));
        assert_eq!(iter.next(), Some(&"test"));
        let val = iter.next();
        assert_eq!(val, Some(&"hallo"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iterator() {
        let mut list = LinkedList::new();
        list.push_front("hallo");
        list.push_front("test");
        list.push_front("nice");
        let vec = list.iter().collect::<Vec<_>>();
        assert_eq!(vec[0], &"nice");
        assert_eq!(vec[1], &"test");
        assert_eq!(vec[2], &"hallo");
        assert_eq!(vec.get(3), None);
    }

    #[test]
    fn get_large_number() {
        let mut list = LinkedList::new();
        for i in 0..1000000 {
            list.push_front(i);
        }
        assert_eq!(list.get(999999), Some(&0));
    }
}
