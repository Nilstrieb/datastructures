#[cfg(test)]
mod test;

use std::fmt::{Debug, Formatter};
use std::hash::Hasher;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::mem;
use std::mem::MaybeUninit;
use std::option::Option::Some;
use std::ptr::NonNull;

fn allocate_nonnull<T>(element: T) -> NonNull<T> {
    // SAFETY: box is always non-null
    unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(element))) }
}

///
/// A more efficient implementation of a linked list
///
/// The packed linked list also consists of nodes, but each node contains several values, that may be copied
/// around for inserts, like in a `Vec`. These nodes are a lot smaller, so the copies should be fairly cheap,
/// trying to use the advantages of lists and mitigating the disadvantages of them (larger memory footprint,
/// no cache locality) by grouping several values together.
///
/// Another way to optimize a linked list is by having a `Vec` of nodes that each have relative references,
/// but this implementation does not implement this.
#[derive(Eq)]
pub struct PackedLinkedList<T, const COUNT: usize> {
    first: Option<NonNull<Node<T, COUNT>>>,
    last: Option<NonNull<Node<T, COUNT>>>,
    len: usize,
    _maker: PhantomData<T>,
}

impl<T, const COUNT: usize> Drop for PackedLinkedList<T, COUNT> {
    fn drop(&mut self) {
        let mut item = self.first;
        while let Some(node) = item {
            let boxed = unsafe { Box::from_raw(node.as_ptr()) };
            item = boxed.next;
        }
    }
}

impl<T, const COUNT: usize> PackedLinkedList<T, COUNT> {
    /// Constructs an empty PackedLinkedList
    pub fn new() -> Self {
        Self {
            first: None,
            last: None,
            len: 0,
            _maker: PhantomData,
        }
    }

    /// The length of the list (O(1))
    pub fn len(&self) -> usize {
        self.len
    }

    // Whether the list is empty (O(1))
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Pushes a new value to the front of the list
    pub fn push_front(&mut self, element: T) {
        // SAFETY: All pointers should always point to valid memory,
        unsafe {
            match self.first {
                None => {
                    self.insert_node_start();
                    self.first.unwrap().as_mut().push_front(element)
                }
                Some(node) if node.as_ref().is_full() => {
                    self.insert_node_start();
                    self.first.unwrap().as_mut().push_front(element)
                }
                Some(mut node) => node.as_mut().push_front(element),
            }
            self.len += 1;
        }
    }

    /// Pushes a new value to the back of the list
    pub fn push_back(&mut self, element: T) {
        // SAFETY: All pointers should always point to valid memory,
        unsafe {
            match self.last {
                None => {
                    self.insert_node_end();
                    self.last.unwrap().as_mut().push_back(element)
                }
                Some(node) if node.as_ref().is_full() => {
                    self.insert_node_end();
                    self.last.unwrap().as_mut().push_back(element)
                }
                Some(mut node) => node.as_mut().push_back(element),
            }
            self.len += 1;
        }
    }

    /// Pops the front element and returns it
    pub fn pop_front(&mut self) -> Option<T> {
        let first = &mut self.first?;
        unsafe {
            let node = first.as_mut();
            debug_assert_ne!(node.size, 0);

            let item = mem::replace(&mut node.values[0], MaybeUninit::uninit()).assume_init();

            if node.size == 1 {
                // the last item, deallocate it
                let mut boxed = Box::from_raw(first.as_ptr());
                if let Some(next) = boxed.next.as_mut() {
                    next.as_mut().prev = None;
                }
                self.first = boxed.next;
                if self.first.is_none() {
                    // if this node was the last one, also remove it from the tail pointer
                    self.last = None;
                }
            } else {
                // more items, move them down
                std::ptr::copy(
                    &node.values[1] as *const _,
                    &mut node.values[0] as *mut _,
                    node.size - 1,
                );
                node.size -= 1;
            }

            self.len -= 1;
            Some(item)
        }
    }

    /// Pops the back value and returns it
    pub fn pop_back(&mut self) -> Option<T> {
        let last = &mut self.last?;
        unsafe {
            let node = last.as_mut();
            debug_assert_ne!(node.size, 0);

            let item =
                mem::replace(&mut node.values[node.size - 1], MaybeUninit::uninit()).assume_init();

            if node.size == 1 {
                // the last item, deallocate it
                let mut boxed = Box::from_raw(last.as_ptr());
                if let Some(previous) = boxed.prev.as_mut() {
                    previous.as_mut().next = None;
                }
                self.last = boxed.prev;
                if self.last.is_none() {
                    // if this node was the last one, also remove it from the tail pointer
                    self.first = None;
                }
            } else {
                // more items
                node.size -= 1;
            }
            self.len -= 1;
            Some(item)
        }
    }

    pub fn cursor_front(&self) -> Cursor<T, COUNT> {
        Cursor {
            node: self.first,
            index: 0,
            list: self,
        }
    }

    pub fn cursor_back(&self) -> Cursor<T, COUNT> {
        Cursor {
            node: self.last,
            // point to the last element in the last node, or 0 if no node is found
            index: self
                .last
                .map(|last| unsafe { last.as_ref().size - 1 })
                .unwrap_or(0),
            list: self,
        }
    }

    pub fn cursor_mut_front(&mut self) -> CursorMut<T, COUNT> {
        CursorMut {
            node: self.first,
            index: 0,
            list: self,
        }
    }

    pub fn cursor_mut_back(&mut self) -> CursorMut<T, COUNT> {
        CursorMut {
            node: self.last,
            // point to the last element in the last node, or 0 if no node is found
            index: self
                .last
                .map(|last| unsafe { last.as_ref().size - 1 })
                .unwrap_or(0),
            list: self,
        }
    }

    pub fn iter(&self) -> iter::Iter<T, COUNT> {
        iter::Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> iter::IterMut<T, COUNT> {
        iter::IterMut::new(self)
    }

    fn insert_node_start(&mut self) {
        let node = Some(allocate_nonnull(Node::new(None, self.first)));
        if let Some(first) = self.first.as_mut() {
            unsafe { first.as_mut().prev = node };
        }
        self.first = node;
        if self.last.is_none() {
            self.last = node;
        }
    }

    fn insert_node_end(&mut self) {
        let node = Some(allocate_nonnull(Node::new(self.last, None)));
        if let Some(last) = self.last.as_mut() {
            unsafe { last.as_mut().next = node };
        }
        self.last = node;
        if self.first.is_none() {
            self.first = node;
        }
    }
}

impl<T, const COUNT: usize> IntoIterator for PackedLinkedList<T, COUNT> {
    type Item = T;
    type IntoIter = iter::IntoIter<Self::Item, COUNT>;

    fn into_iter(self) -> Self::IntoIter {
        iter::IntoIter::new(self)
    }
}

impl<T, const COUNT: usize> FromIterator<T> for PackedLinkedList<T, COUNT> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = PackedLinkedList::new();
        for item in iter {
            list.push_back(item);
        }
        list
    }
}

impl<T, const COUNT: usize> Extend<T> for PackedLinkedList<T, COUNT> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_back(item);
        }
    }
}

impl<T: std::fmt::Debug, const COUNT: usize> std::fmt::Debug for PackedLinkedList<T, COUNT> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T, const COUNT: usize> Default for PackedLinkedList<T, COUNT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, const COUNT: usize> Clone for PackedLinkedList<T, COUNT> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T: std::hash::Hash, const COUNT: usize> std::hash::Hash for PackedLinkedList<T, COUNT> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iter().for_each(|item| item.hash(state))
    }
}

impl<T: PartialEq, const COUNT: usize> PartialEq for PackedLinkedList<T, COUNT> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

/// A single node in the packed linked list
///
/// The node can have 1 to `COUNT` items.
/// A node is never guaranteed to be full, even if it has a next node
/// A node is always guaranteed to be non-empty
struct Node<T, const COUNT: usize> {
    prev: Option<NonNull<Node<T, COUNT>>>,
    next: Option<NonNull<Node<T, COUNT>>>,
    values: [MaybeUninit<T>; COUNT],
    size: usize,
}

impl<T: Debug, const COUNT: usize> Debug for Node<T, COUNT> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("prev", &self.prev)
            .field("next", &self.next)
            .field("values", &{
                let mut str = String::from("[");
                for i in 0..self.size {
                    str.push_str(&format!("{:?}, ", unsafe { &*self.values[i].as_ptr() }))
                }
                for _ in self.size..COUNT {
                    str.push_str("(uninit), ")
                }
                str.push(']');
                str
            })
            .field("size", &self.size)
            .finish()
    }
}

impl<T, const COUNT: usize> Node<T, COUNT> {
    fn new(prev: Option<NonNull<Node<T, COUNT>>>, next: Option<NonNull<Node<T, COUNT>>>) -> Self {
        Self {
            prev,
            next,
            // SAFETY: This is safe because we claim that the MaybeUninits are initialized, which they always are,
            // since any uninitialized memory is a valid MaybeUninit
            values: unsafe { MaybeUninit::uninit().assume_init() },
            size: 0,
        }
    }

    /// Checks whether the node is full
    fn is_full(&self) -> bool {
        self.size == COUNT
    }

    /// Pushes a new value to the back
    /// # Safety
    /// The node must not be full
    unsafe fn push_back(&mut self, element: T) {
        debug_assert!(self.size < COUNT);
        self.values[self.size] = MaybeUninit::new(element);
        self.size += 1;
    }

    /// Pushes a new value to the front
    /// # Safety
    /// The node must not be full
    unsafe fn push_front(&mut self, element: T) {
        debug_assert!(self.size < COUNT);
        // copy all values up
        if COUNT > 1 {
            std::ptr::copy(
                &self.values[0] as *const _,
                &mut self.values[1] as *mut _,
                self.size,
            );
        }

        self.values[0] = MaybeUninit::new(element);
        self.size += 1;
    }

    /// Inserts a new value at the index, copying the values up
    /// # Safety
    /// The node must not be full and the index must not be out of bounds
    /// This function should not be called on an empty node, use `push_back` instead
    unsafe fn insert(&mut self, element: T, index: usize) {
        debug_assert!(self.size < COUNT);
        debug_assert!(self.size > index);
        // copy all values up
        for i in (index..self.size).rev() {
            println!("{}", i);
            self.values[i + 1] = mem::replace(&mut self.values[i], MaybeUninit::uninit());
        }
        self.values[index] = MaybeUninit::new(element);
        self.size += 1;
    }
}

macro_rules! implement_cursor {
    ($cursor:ident) => {
        impl<'a, T, const COUNT: usize> $cursor<'a, T, COUNT> {
            pub fn get(&self) -> Option<&T> {
                self.node
                    .map(|nn| unsafe { nn.as_ref().values[self.index].as_ptr().as_ref().unwrap() })
            }

            pub fn move_next(&mut self) {
                match self.node {
                    None => {
                        // currently on the ghost node, move to the first node
                        self.node = self.list.first;
                        self.index = 0;
                    }
                    Some(node) => unsafe {
                        let node = node.as_ref();
                        if self.index == node.size - 1 {
                            // the last item, go to the next node
                            self.node = node.next;
                            self.index = 0;
                        } else {
                            // stay on the same node
                            self.index += 1;
                        }
                    },
                }
            }
            pub fn move_prev(&mut self) {
                match self.node {
                    None => {
                        // currently on the ghost node, move to the first node
                        self.node = self.list.last;
                        self.index = self
                            .list
                            .last
                            .map(|nn| unsafe { nn.as_ref().size - 1 })
                            .unwrap_or(0);
                    }
                    Some(node) => unsafe {
                        let node = node.as_ref();
                        if self.index == 0 {
                            // the first item, go to the previous node
                            self.node = node.prev;
                            self.index = node.prev.map(|nn| nn.as_ref().size - 1).unwrap_or(0);
                        } else {
                            // stay on the same node
                            self.index -= 1;
                        }
                    },
                }
            }
        }
    };
}

/// A cursor for navigating the Packed Linked List
pub struct Cursor<'a, T, const COUNT: usize> {
    node: Option<NonNull<Node<T, COUNT>>>,
    index: usize,
    list: &'a PackedLinkedList<T, COUNT>,
}

// A cursor for navigating and editing the Packed Linked List
pub struct CursorMut<'a, T, const COUNT: usize> {
    node: Option<NonNull<Node<T, COUNT>>>,
    index: usize,
    list: &'a mut PackedLinkedList<T, COUNT>,
}

implement_cursor!(Cursor);
implement_cursor!(CursorMut);

impl<'a, T, const COUNT: usize> CursorMut<'a, T, COUNT> {
    pub fn get_mut(&mut self) -> Option<&mut T> {
        let index = self.index;
        self.node
            .as_mut()
            .map(|nn| unsafe { nn.as_mut().values[index].as_mut_ptr().as_mut().unwrap() })
    }

    pub fn replace(&mut self, _element: T) -> Option<T> {
        todo!()
    }

    pub fn remove(&mut self) -> Option<T> {
        todo!()
    }

    /// Inserts a new element after the element this cursor is pointing to.  
    /// If the cursor is pointing at the ghost node, the item gets inserted at the start of the list  
    /// The cursor position will not change.  
    pub fn insert_after(&mut self, element: T) {
        match self.node {
            None => self.list.push_front(element),
            Some(mut current_node) => {
                let current = unsafe { current_node.as_mut() };

                // if we point at the last element, we do not need to copy anything
                let append = self.index == current.size - 1;
                // There are several cases here
                // 1. we append an item to the node, and it is not full
                // 2. we append an item to the node, and it is full
                // 3. we insert an item into the node, and it is not full
                // 4. we insert an item into the node, and it is full
                match (append, current.is_full()) {
                    (true, false) => {
                        // SAFETY: the node is not full
                        unsafe { current.push_back(element) };
                    }
                    (true, true) => {
                        // check whether the next node is full. if it is not full, insert it at the start
                        // if it is full or the next node doesn't exist, allocate a new node inbetween
                        let next_node = unsafe { current.next.as_mut().map(|nn| nn.as_mut()) };
                        let need_allocate = next_node
                            .as_ref()
                            .map(|node| node.is_full())
                            .unwrap_or(true);

                        if need_allocate {
                            unsafe {
                                let mut new_node = self.allocate_new_node_after();
                                new_node.as_mut().push_back(element);
                            }
                        } else {
                            let next_node = next_node
                                .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
                            // SAFETY: the node is not full, because `need_allocate` is false
                            unsafe { next_node.push_back(element) };
                        }
                    }
                    // SAFETY: the node is not full and the index is not out of bounds
                    (false, false) => unsafe { current.insert(element, self.index + 1) },
                    (false, true) => {
                        // we need to copy some values to the next node, always allocate a new one to avoid needing to copy too many values
                        // nodes that are not very full will make insertions faster later, so we prefer them
                        // this is a bad though if we repeatedly insert at the same position here, so maybe we want to insert it into the next node anyways
                        unsafe {
                            let mut next = self.allocate_new_node_after();
                            let mut next = next.as_mut();
                            // example: current node of COUNT=8 is full, we want to insert at 7
                            // self.index=6
                            // copy 2 values to the next node, 7 & 8
                            let to_copy = current.size - self.index;
                            std::ptr::copy_nonoverlapping(
                                current.values[self.index + 1].as_ptr(),
                                next.values[0].as_mut_ptr(),
                                to_copy,
                            );
                            //for i in self.index..5 {
                            //
                            //}
                            current.values[self.index + 1] = MaybeUninit::new(element);
                            next.size = to_copy;
                            current.size = self.index + 2;
                        }
                    }
                }
                self.list.len += 1;
            }
        }
    }

    pub fn insert_before(&mut self, _element: T) {}

    /// allocates a new node after the cursor
    /// if self.node is None, it allocates the node at the start of the list
    /// # Safety
    /// The node must immediately be filled with at least on element, since an empty node is not a valid state
    unsafe fn allocate_new_node_after(&mut self) -> NonNull<Node<T, COUNT>> {
        let mut new_node = allocate_nonnull(Node::new(
            self.node, None, // will be replaced in the match below
        ));

        match self.node {
            None => {
                match self.list.first {
                    None => self.list.last = Some(new_node),
                    Some(mut first) => first.as_mut().prev = Some(new_node),
                }
                new_node.as_mut().next = self.list.first;
                self.list.first = Some(new_node);
            }
            Some(mut node) => {
                new_node.as_mut().next = node.as_ref().next;
                node.as_mut().next = Some(new_node);
            }
        }
        new_node
    }
}

mod iter {
    use super::{Node, PackedLinkedList};
    use std::marker::PhantomData;
    use std::mem;
    use std::mem::MaybeUninit;
    use std::ptr::NonNull;

    #[derive(Debug)]
    pub struct Iter<'a, T, const COUNT: usize> {
        node: Option<&'a Node<T, COUNT>>,
        index: usize,
    }

    impl<'a, T, const COUNT: usize> Iter<'a, T, COUNT> {
        pub(super) fn new(list: &'a PackedLinkedList<T, COUNT>) -> Self {
            Self {
                node: list.first.as_ref().map(|nn| unsafe { nn.as_ref() }),
                index: 0,
            }
        }
    }

    impl<'a, T, const COUNT: usize> Iterator for Iter<'a, T, COUNT> {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            let node = self.node?;
            // SAFETY: assume that all pointers point to the correct nodes,
            // and that the sizes of the nodes are set correctly
            unsafe {
                if node.size > self.index {
                    // take more
                    let item = node.values[self.index].as_ptr().as_ref().unwrap();
                    self.index += 1;
                    Some(item)
                } else {
                    // next node
                    let next_node = node.next.as_ref()?.as_ref();
                    self.index = 1;
                    self.node = Some(next_node);
                    // a node should never be empty
                    debug_assert_ne!(next_node.size, 0);
                    Some(next_node.values[0].as_ptr().as_ref().unwrap())
                }
            }
        }
    }

    #[derive(Debug)]
    pub struct IterMut<'a, T, const COUNT: usize> {
        node: Option<NonNull<Node<T, COUNT>>>,
        index: usize,
        _marker: PhantomData<&'a T>,
    }

    impl<'a, T, const COUNT: usize> IterMut<'a, T, COUNT> {
        pub(super) fn new(list: &'a mut PackedLinkedList<T, COUNT>) -> Self {
            Self {
                node: list.first,
                index: 0,
                _marker: PhantomData,
            }
        }
    }

    impl<'a, T: 'a, const COUNT: usize> Iterator for IterMut<'a, T, COUNT> {
        type Item = &'a mut T;

        fn next(&mut self) -> Option<Self::Item> {
            // SAFETY: assume that all pointers point to the correct nodes,
            // and that the sizes of the nodes are set correctly
            unsafe {
                let mut node = self.node?;
                let node = node.as_mut();
                if node.size > self.index {
                    // take more
                    let ptr = node.values[self.index].as_ptr() as *mut T;
                    let item = ptr.as_mut().unwrap();
                    self.index += 1;

                    Some(item)
                } else {
                    // next node
                    let mut next_node = node.next?;
                    debug_assert_ne!(next_node.as_ref().size, 0);
                    self.index = 1;
                    self.node = Some(next_node);
                    // a node should never be empty
                    let ptr = next_node.as_mut().values[0].as_ptr() as *mut T;
                    Some(ptr.as_mut().unwrap())
                }
            }
        }
    }

    #[derive(Debug)]
    pub struct IntoIter<T, const COUNT: usize> {
        node: Option<Box<Node<T, COUNT>>>,
        index: usize,
    }

    impl<T, const COUNT: usize> Drop for IntoIter<T, COUNT> {
        fn drop(&mut self) {
            for _ in self {}
        }
    }

    impl<T, const COUNT: usize> IntoIter<T, COUNT> {
        pub(super) fn new(list: PackedLinkedList<T, COUNT>) -> Self {
            let iter = Self {
                node: list.first.map(|nn| unsafe { Box::from_raw(nn.as_ptr()) }),
                index: 0,
            };
            // do not drop the list, the iterator has taken 'ownership'
            mem::forget(list);
            iter
        }
    }

    impl<T, const COUNT: usize> Iterator for IntoIter<T, COUNT> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            // take the node. the node has to either be returned or replaced by a new one. the None left
            // behind here is *not* a valid state
            let mut node = self.node.take()?;

            // SAFETY: see more detailed comments
            unsafe {
                if node.size > self.index {
                    // take more items from the node
                    // take out the item and replace it with uninitialized memory
                    // the index pointer is increased, so no one will access this again
                    let item = mem::replace(&mut node.values[self.index], MaybeUninit::uninit())
                        .assume_init();
                    self.index += 1;
                    // re-insert the node
                    self.node = Some(node);
                    Some(item)
                } else {
                    // go to the next node
                    // if next is empty, return None and stop the iteration
                    // take ownership over the node. the last node will be dropped here
                    let mut next_node = Box::from_raw(node.next?.as_ptr());
                    next_node.prev = None;
                    self.index = 1;
                    // a node should never be empty
                    debug_assert_ne!(next_node.size, 0);
                    self.node = Some(next_node);
                    // see comment above
                    Some(
                        mem::replace(
                            &mut self.node.as_mut().unwrap().values[0],
                            MaybeUninit::uninit(),
                        )
                        .assume_init(),
                    )
                }
            }
        }
    }
}
