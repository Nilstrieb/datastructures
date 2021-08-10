#[cfg(test)]
mod test;

use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
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
/// let mut node = list.front_node_mut().unwrap();
/// node.push_after(3);
/// node.push_after(2);
/// let next = node.next().unwrap();
/// let next = next.next().unwrap();
/// assert_eq!(*next.get(), 3);
/// ```
///
/// # Note
/// You should generally not use Linked Lists, and if you really do need to use one, use `std::collections::LinkedList`
#[derive(Eq)]
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

    /// Push an element to the start of the list, O(1)
    pub fn push_front(&mut self, element: T) {
        let new_node = allocate_nonnull(Node {
            value: element,
            next: self.start,
            prev: None,
        });
        match self.start {
            Some(mut old_start) => {
                // SAFETY: All pointers should always be valid.
                unsafe { old_start.as_mut() }.prev = Some(new_node);
            }
            // List is empty - set the end
            None => self.end = Some(new_node),
        }
        self.start = Some(new_node);
    }

    /// Push an element to the end of the list, O(1)
    pub fn push_back(&mut self, element: T) {
        let new_node = allocate_nonnull(Node {
            value: element,
            next: None,
            prev: self.end,
        });
        match self.end {
            Some(mut old_end) => {
                // SAFETY: All pointers should always be valid.
                unsafe { old_end.as_mut() }.next = Some(new_node);
            }
            // List is empty - set the start
            None => self.start = Some(new_node),
        }
        self.end = Some(new_node);
    }

    /// Pops the first value in the list and returns it, O(1)
    pub fn pop_front(&mut self) -> Option<T> {
        self.start.map(|node| {
            // SAFETY: all pointers should always be valid
            let boxed = unsafe { Box::from_raw(node.as_ptr()) };
            self.start = boxed.next;
            match boxed.next {
                Some(mut next) => {
                    // the next item is now the first item
                    unsafe { next.as_mut().prev = None }
                }
                // node was the last element in the list
                None => self.end = None,
            }
            boxed.value
            // node is freed here
        })
    }

    /// Pops the last value in the list and returns it, O(1)
    pub fn pop_back(&mut self) -> Option<T> {
        self.end.map(|node| {
            // SAFETY: all pointers should always be valid
            let boxed = unsafe { Box::from_raw(node.as_ptr()) };
            self.end = boxed.prev;
            match boxed.prev {
                Some(mut prev) => {
                    // the previous item is now the last item
                    unsafe { prev.as_mut().next = None }
                }
                // node was the last element in the list
                None => self.start = None,
            }
            boxed.value
            // node is freed here
        })
    }

    /// Get an element from the list, O(n)
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

    /// Gets the last element from the list, O(1)
    pub fn get_tail(&self) -> Option<&T> {
        self.end.as_ref().map(|nn| unsafe { &nn.as_ref().value })
    }

    /// Gets the first element from the list, O(1)
    pub fn get_head(&self) -> Option<&T> {
        self.start.as_ref().map(|nn| unsafe { &nn.as_ref().value })
    }

    /// Get a node from the list that can only be used for navigation, O(n)
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

    /// Get the head node from the list that can only be used for navigation
    pub fn front_node(&self) -> Option<&Node<T>> {
        self.start.as_ref().map(|nn| unsafe { nn.as_ref() })
    }

    /// Get the tail node from the list that can only be used for navigation
    pub fn back_node(&self) -> Option<&Node<T>> {
        self.end.as_ref().map(|nn| unsafe { nn.as_ref() })
    }
    /// Get the head node from the list that can be used the edit the list
    pub fn front_node_mut(&mut self) -> Option<&mut Node<T>> {
        self.start.as_mut().map(|nn| unsafe { nn.as_mut() })
    }

    /// Get the tail node from the list that can be used the edit the list
    pub fn back_node_mut(&mut self) -> Option<&mut Node<T>> {
        self.end.as_mut().map(|nn| unsafe { nn.as_mut() })
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

    /// Returns a mut iterator over the items
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut::new(self)
    }

    /// Returns an iterator owning the items
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter::new(self)
    }
}

/////
///// std trait implementations
/////

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

impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T: Hash> Hash for LinkedList<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iter().for_each(|item| item.hash(state));
    }
}

impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        // TODO this is very inefficient
        if self.len() != other.len() {
            return false;
        }
        self.iter()
            .zip(other.iter())
            .all(|(left, right)| left == right)
    }
}

impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let mut list = Self::new();
        while let Some(item) = iter.next() {
            list.push_back(item)
        }
        list
    }
}

impl<T> Extend<T> for LinkedList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();
        while let Some(item) = iter.next() {
            self.push_back(item)
        }
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
}

fn allocate_nonnull<T>(element: T) -> NonNull<T> {
    let boxed = Box::new(element);
    // SAFETY: box is always non-null
    unsafe { NonNull::new_unchecked(Box::leak(boxed)) }
}

/// The iterator over the linked list
pub struct Iter<'a, T>(Option<&'a Node<T>>);

impl<'a, T> Iter<'a, T> {
    fn new(list: &'a LinkedList<T>) -> Self {
        Self(list.start.as_ref().map(|nn| {
            // SAFETY: All pointers should always be valid, the list lives as long as its items
            unsafe { nn.as_ref() }
        }))
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.0;
        match current {
            Some(node) => {
                self.0 = node.next.as_ref().map(|nn| {
                    // SAFETY: All pointers should always be valid
                    unsafe { nn.as_ref() }
                });
                Some(&node.value)
            }
            None => None,
        }
    }
}

/// The owning iterator over the linked list
pub struct IntoIter<T>(Option<Box<Node<T>>>);

impl<T> IntoIter<T> {
    fn new(list: LinkedList<T>) -> Self {
        let iter = Self(list.start.as_ref().map(|nn| {
            // SAFETY: All pointers should always be valid, the list lives as long as its items
            unsafe { Box::from_raw(nn.as_ptr()) }
        }));
        // We are not allowed to drop the list - the items will be freed during the iteration
        std::mem::forget(list);
        iter
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        while let Some(_) = self.next() {}
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.0.take();
        match current {
            Some(node) => {
                self.0 = node.next.as_ref().map(|nn| {
                    // SAFETY: All pointers should always be valid, the list lives as long as its items
                    unsafe { Box::from_raw(nn.as_ptr()) }
                });
                Some(node.value)

                // the node is freed here
            }
            None => None,
        }
    }
}

/// The iterator over the linked list
pub struct IterMut<'a, T>(Option<&'a mut Node<T>>);

impl<'a, T> IterMut<'a, T> {
    fn new(list: &'a mut LinkedList<T>) -> Self {
        Self(list.start.as_mut().map(|nn| {
            // SAFETY: All pointers should always be valid, the list lives as long as its items
            unsafe { nn.as_mut() }
        }))
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.0.take();
        match current {
            Some(node) => {
                self.0 = node.next.as_mut().map(|nn| {
                    // SAFETY: All pointers should always be valid
                    unsafe { nn.as_mut() }
                });
                Some(&mut node.value)
            }
            None => None,
        }
    }
}
