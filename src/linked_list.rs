use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A doubly linked list using unsafe code.  
/// It is loosely inspired by the `std::collections::LinkedList`, but I haven't looked at that one too close,
/// so most it is my own.
///
/// It was not made with efficiency in mind, but at least it doesn't but `std::rc::Rc` everywhere, but uses
/// unsafe pointers instead.
///
/// # How to use
/// ```
/// # use datastructures::linked_list::LinkedList;
/// #
/// let mut list = LinkedList::new();
/// list.push_front("hello");
/// assert_eq!(list.get(0), Some(&"hello"));
/// list.push_back("bye");
/// assert_eq!(list.get(1), Some(&"bye"));
/// ```
///
/// The list can also be edited using the `Node` methods
/// ```
/// # use datastructures::linked_list::LinkedList;
/// #
/// let mut list = LinkedList::new();
///
/// list.push_front(1);
/// let mut node = list.get_mut_head_node().unwrap();
/// node.push_after(3);
/// node.push_after(2);
/// let next = node.next().unwrap();
/// let next = next.next().unwrap();
/// assert_eq!(*next.get(), 3);
/// ```
///
/// # Note
/// You should generally not use Linked Lists, and if you really do need to use one, use `std::collections::LinkedList`
pub struct LinkedList<T> {
    start: Option<NonNull<Node<T>>>,
    end: Option<NonNull<Node<T>>>,
    _marker: PhantomData<T>,
}

impl<T> LinkedList<T> {
    /// Creates a new empty Linked List
    pub fn new() -> LinkedList<T> {
        Self {
            start: None,
            end: None,
            _marker: PhantomData,
        }
    }

    /// Push an element to the start of the list (O(1))
    pub fn push_front(&mut self, element: T) {
        match self.start {
            // empty list
            None => {
                let new_node = allocate_nonnull(Node {
                    value: element,
                    next: None,
                    prev: None,
                });
                self.start = Some(new_node);
                self.end = Some(new_node);
            }
            // at lest one element
            Some(mut old_start) => {
                let new_node = allocate_nonnull(Node {
                    value: element,
                    next: Some(old_start),
                    prev: None,
                });
                // SAFETY: All pointers should always be valid
                unsafe { old_start.as_mut() }.prev = Some(new_node);
                self.start = Some(new_node);
            }
        }
    }

    /// Push an element to the end of the list (O(1))
    pub fn push_back(&mut self, element: T) {
        match self.end {
            None => {
                let new_node = allocate_nonnull(Node {
                    value: element,
                    next: None,
                    prev: None,
                });
                self.start = Some(new_node);
                self.end = Some(new_node);
            }
            Some(mut old_end) => {
                let new_node = allocate_nonnull(Node {
                    value: element,
                    next: None,
                    prev: Some(old_end),
                });
                // SAFETY: All pointers should always be valid
                unsafe { old_end.as_mut() }.next = Some(new_node);
                self.end = Some(new_node);
            }
        }
    }

    /// Get an element from the list (O(n))
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

    /// Gets the last element from the list (O(1))
    pub fn get_tail(&self) -> Option<&T> {
        self.end.as_ref().map(|nn| unsafe { &nn.as_ref().value })
    }

    /// Gets the first element from the list (O(1))
    pub fn get_head(&self) -> Option<&T> {
        self.start.as_ref().map(|nn| unsafe { &nn.as_ref().value })
    }

    /// Get a node from the list that can only be used for navigation
    pub fn get_node(&self, mut index: usize) -> Option<&Node<T>> {
        let mut node = &self.start;
        let mut result = None;
        while let Some(content) = node {
            // SAFETY: All pointers should always be valid
            let content = unsafe { content.as_ref() };
            if index == 0 {
                result = Some(content);
                break;
            }
            index -= 1;
            node = &content.next;
        }
        result
    }

    /// Get the head node from the list that can only be used for navigation
    pub fn get_head_node(&self) -> Option<&Node<T>> {
        self.start.as_ref().map(|nn| unsafe { nn.as_ref() })
    }

    /// Get the tail node from the list that can only be used for navigation
    pub fn get_tail_node(&self) -> Option<&Node<T>> {
        self.end.as_ref().map(|nn| unsafe { nn.as_ref() })
    }
    /// Get the head node from the list that can be used the edit the list
    pub fn get_mut_head_node(&mut self) -> Option<&mut Node<T>> {
        self.start.as_mut().map(|nn| unsafe { nn.as_mut() })
    }

    /// Get the tail node from the list that can be used the edit the list
    pub fn get_mut_tail_node(&mut self) -> Option<&mut Node<T>> {
        self.end.as_mut().map(|nn| unsafe { nn.as_mut() })
    }

    /// Get a node from the list that can be used the edit the list
    pub fn get_mut_node(&mut self, mut index: usize) -> Option<&mut Node<T>> {
        let mut node = &mut self.start;
        let mut result = None;
        while let Some(ref mut content) = node {
            // SAFETY: All pointers should always be valid
            let content = unsafe { content.as_mut() };
            if index == 0 {
                result = Some(content);
                break;
            }
            index -= 1;
            node = &mut content.next;
        }
        result
    }

    /// Calculates the length of the list
    /// # Important
    /// This implementation is O(n), since unlike in `std::collections::LinkedList`, the length of the list is not stored
    /// (and can't be because the list can be modified through nodes - a node could theoretically have a reference to the list,
    /// but that would make node extraction slower because you'd always have to construct a new struct.
    pub fn len(&self) -> usize {
        self.iter().count()
    }

    /// Returns an iterator over the items
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }
}

impl<T: Debug> Debug for LinkedList<T> {
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
/// Can be used to navigate the `LinkedList`, using the `Node::get_next` and `Node::get_previous` methods,
/// and edit the List using the push methods.
///
/// # Examples
/// ```
/// # use datastructures::linked_list::*;
/// #
/// let mut list = LinkedList::new();
/// list.push_front(1);
/// let mut node = list.get_mut_node(0);
/// ```
///
#[derive(Debug)]
pub struct Node<T> {
    value: T,
    next: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
}

impl<T> Node<T> {
    /// Push a value after this node
    pub fn push_after(&mut self, element: T) {
        let new_node = Some(allocate_nonnull(Node {
            value: element,
            next: self.next,
            prev: NonNull::new(self as _),
        }));
        self.next.map(|mut next| {
            // SAFETY: All pointers should always be valid and created from a box
            unsafe { next.as_mut() }.prev = new_node
        });
        self.next = new_node;
    }

    /// Push a value before this node
    pub fn push_before(&mut self, element: T) {
        let new_node = Some(allocate_nonnull(Node {
            value: element,
            next: NonNull::new(self as _),
            prev: self.prev,
        }));
        self.prev.map(|mut next| {
            // SAFETY: All pointers should always be valid and created from a box
            unsafe { next.as_mut() }.next = new_node
        });
        self.prev = new_node;
    }

    /// Get the next node
    pub fn next(&self) -> Option<&Node<T>> {
        self.next.as_ref().map(|nn| unsafe { nn.as_ref() })
    }

    /// Get the next node mutably
    pub fn next_mut(&mut self) -> Option<&mut Node<T>> {
        self.next.as_mut().map(|nn| unsafe { nn.as_mut() })
    }

    /// Get the previous node
    pub fn previous(&self) -> Option<&Node<T>> {
        self.prev.as_ref().map(|nn| unsafe { nn.as_ref() })
    }

    /// Get the previous node mutably
    pub fn previous_mut(&mut self) -> Option<&mut Node<T>> {
        self.prev.as_mut().map(|nn| unsafe { nn.as_mut() })
    }

    /// Gets the value from the node
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Gets the value from the node
    pub fn set(&mut self, value: T) {
        self.value = value;
    }

    /// Gets the value from the node and replaces it with the old one
    pub fn replace_value(&mut self, value: T) -> T {
        std::mem::replace(&mut self.value, value)
    }

    /// Removes a value from the List and returns it
    pub fn remove(&mut self) -> T {
        // SAFETY: All pointers should always be valid
        unsafe {
            self.next.map(|mut next| next.as_mut().prev = self.prev);
            self.prev.map(|mut prev| prev.as_mut().next = self.next);
        }

        // SAFETY: A reference is always valid and we have the only one now
        let node = unsafe { Box::from_raw(self) };
        node.value
    }
}

fn allocate_nonnull<T>(element: T) -> NonNull<T> {
    let boxed = Box::new(element);
    // SAFETY: box is always non-null
    unsafe { NonNull::new_unchecked(Box::leak(boxed)) }
}

/// The iterator over the linked list
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

impl<'a, T> Iterator for Iter<'a, T> {
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
    fn push_start_end() {
        let mut list = LinkedList::new();
        list.push_back(3);
        list.push_front(2);
        list.push_front(1);
        list.push_back(4);
        list.push_back(5);
        let vec = list.iter().cloned().collect::<Vec<_>>();
        assert_eq!(&vec[..], &[1, 2, 3, 4, 5]);
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

    #[test]
    fn node_operations() {
        let mut list = LinkedList::new();
        list.push_front(1);
        list.push_back(2);
        {
            let node = list.get_mut_node(1).unwrap();
            assert_eq!(*node.get(), 2);
            node.push_after(4);
            let next = node.next_mut().unwrap();
            assert!(matches!(next.next(), None));
            next.push_before(3)
        }
        let vec = list.iter().cloned().collect::<Vec<_>>();
        assert_eq!(&vec[..], &[1, 2, 3, 4]);
    }

    #[test]
    fn node_values() {
        let mut list = LinkedList::new();
        list.push_front(1);
        let node = list.get_mut_node(0).unwrap();
        assert_eq!(*node.get(), 1);
        assert_eq!(node.replace_value(2), 1);
        assert_eq!(*node.get(), 2);
        node.push_after(3);
        let node = node.next_mut().unwrap();
        node.set(4);
        assert_eq!(*node.get(), 4);
    }

    #[test]
    fn node_removal() {
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(4);
        let node_two = list.get_mut_head_node().unwrap().next_mut().unwrap();
        node_two.replace_value(3);
        let three = node_two.remove();
        assert_eq!(three, 3);
        assert_eq!(list.get_head(), Some(&1));
        assert_eq!(list.get_tail(), Some(&4));
        assert_eq!(*list.get_head_node().unwrap().next().unwrap().get(), 4);
    }
}
