#[cfg(test)]
mod test;

use std::fmt::Formatter;
use std::hash::Hasher;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::mem;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

fn allocate_nonnull<T>(element: T) -> NonNull<T> {
    // SAFETY: box is always non-null
    unsafe { NonNull::new_unchecked(Box::leak(Box::new(element))) }
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
                boxed.next.as_mut().map(|next| next.as_mut().prev = None);
                self.first = boxed.next;
                if let None = self.first {
                    // if this node was the last one, also remove it from the tail pointer
                    self.last = None;
                }
            } else {
                // more items, move them down
                std::ptr::copy(
                    &node.values[1] as *const _,
                    &mut node.values[0] as *mut _,
                    node.size,
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
                boxed
                    .prev
                    .as_mut()
                    .map(|previous| previous.as_mut().next = None);
                self.last = boxed.prev;
                if let None = self.last {
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

    pub fn iter(&self) -> Iter<T, COUNT> {
        Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> IterMut<T, COUNT> {
        IterMut::new(self)
    }

    pub fn into_iter(self) -> IntoIter<T, COUNT> {
        IntoIter::new(self)
    }

    fn insert_node_start(&mut self) {
        let node = Some(allocate_nonnull(Node::new(None, self.first)));
        self.first
            .as_mut()
            .map(|first| unsafe { first.as_mut().prev = node });
        self.first = node;
        if let None = self.last {
            self.last = node;
        }
    }

    fn insert_node_end(&mut self) {
        let node = Some(allocate_nonnull(Node::new(self.last, None)));
        self.last
            .as_mut()
            .map(|last| unsafe { last.as_mut().next = node });
        self.last = node;
        if let None = self.first {
            self.first = node;
        }
    }
}

impl<T, const COUNT: usize> FromIterator<T> for PackedLinkedList<T, COUNT> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = PackedLinkedList::new();
        let mut iter = iter.into_iter();
        while let Some(item) = iter.next() {
            list.push_back(item);
        }
        list
    }
}

impl<T, const COUNT: usize> Extend<T> for PackedLinkedList<T, COUNT> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let mut iter = iter.into_iter();
        while let Some(item) = iter.next() {
            self.push_back(item);
        }
    }
}

impl<T, const COUNT: usize> std::fmt::Debug for PackedLinkedList<T, COUNT>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T, const COUNT: usize> Default for PackedLinkedList<T, COUNT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const COUNT: usize> Clone for PackedLinkedList<T, COUNT>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T, const COUNT: usize> std::hash::Hash for PackedLinkedList<T, COUNT>
where
    T: std::hash::Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iter().for_each(|item| item.hash(state))
    }
}

impl<T, const COUNT: usize> PartialEq for PackedLinkedList<T, COUNT>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

/// A single node in the packed linked list
///
/// The node can have 1 to `COUNT` items.
/// A node is never guaranteed to be full, even if it has a next node
/// A node is always guaranteed to be non-empty
#[derive(Debug)]
struct Node<T, const COUNT: usize> {
    prev: Option<NonNull<Node<T, COUNT>>>,
    next: Option<NonNull<Node<T, COUNT>>>,
    values: [MaybeUninit<T>; COUNT],
    size: usize,
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
}

#[derive(Debug)]
pub struct Iter<'a, T, const COUNT: usize> {
    node: Option<&'a Node<T, COUNT>>,
    index: usize,
}

impl<'a, T, const COUNT: usize> Iter<'a, T, COUNT> {
    fn new(list: &'a PackedLinkedList<T, COUNT>) -> Self {
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
    fn new(list: &'a mut PackedLinkedList<T, COUNT>) -> Self {
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
        while let Some(_) = self.next() {}
    }
}

impl<T, const COUNT: usize> IntoIter<T, COUNT> {
    fn new(list: PackedLinkedList<T, COUNT>) -> Self {
        let iter = Self {
            node: list.first.map(|nn| unsafe { Box::from_raw(nn.as_ptr()) }),
            index: 0,
        };
        // do not drop the list
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
                let item =
                    mem::replace(&mut node.values[self.index], MaybeUninit::uninit()).assume_init();
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
