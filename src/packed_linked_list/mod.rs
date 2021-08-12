#[cfg(test)]
mod test;

use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

fn allocate_nonnull<T>(element: T) -> NonNull<T> {
    let boxed = Box::new(element);
    // SAFETY: box is always non-null
    unsafe { NonNull::new_unchecked(Box::leak(boxed)) }
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
pub struct PackedLinkedList<T, const COUNT: usize> {
    first: Option<NonNull<Node<T, COUNT>>>,
    last: Option<NonNull<Node<T, COUNT>>>,
    _maker: PhantomData<T>,
}

impl<T, const COUNT: usize> PackedLinkedList<T, COUNT> {
    pub fn new() -> Self {
        Self {
            first: None,
            last: None,
            _maker: PhantomData,
        }
    }

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
        }
    }

    pub fn iter(&self) -> Iter<T, COUNT> {
        Iter::new(self)
    }

    fn insert_node_start(&mut self) {
        let node = Some(allocate_nonnull(Node::new(None, self.first)));
        self.first
            .as_mut()
            .map(|first| unsafe { first.as_mut().prev = node });
        self.first = node;
    }
}

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
        unsafe {
            std::ptr::copy(
                &self.values[0] as *const _,
                &mut self.values[1] as *mut _,
                self.size,
            )
        }
        self.values[0] = MaybeUninit::new(element);
        self.size += 1;
    }
}

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
