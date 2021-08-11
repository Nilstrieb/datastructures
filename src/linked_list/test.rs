use super::*;

#[test]
fn random_access() {
    let list = create_list(&["nice", "test", "hallo"]);
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
fn pop_back() {
    let mut list = create_list(&["hi", "3", "5"]);
    assert_eq!(Some("5"), list.pop_back());
    assert_eq!(Some("3"), list.pop_back());
    assert_eq!(Some("hi"), list.pop_back());
    assert_eq!(None, list.pop_back());
}

#[test]
fn pop_front() {
    let mut list = create_list(&["hi", "3", "5"]);
    assert_eq!(Some("hi"), list.pop_front());
    assert_eq!(Some("3"), list.pop_front());
    assert_eq!(Some("5"), list.pop_front());
    assert_eq!(None, list.pop_front());
}

#[test]
fn iter_simple() {
    let list = create_list(&["nice", "test", "hallo"]);
    let mut iter = list.iter();
    assert_eq!(iter.next(), Some(&"nice"));
    assert_eq!(iter.next(), Some(&"test"));
    let val = iter.next();
    assert_eq!(val, Some(&"hallo"));
    assert_eq!(iter.next(), None);
}

#[test]
fn iterator() {
    let list = create_list(&["nice", "test", "hallo"]);
    let vec = list.iter().collect::<Vec<_>>();
    assert_eq!(vec[0], &"nice");
    assert_eq!(vec[1], &"test");
    assert_eq!(vec[2], &"hallo");
    assert_eq!(vec.get(3), None);
}

#[test]
fn into_iterator() {
    let list = create_list(&["nice", "test", "hallo"]);
    let vec = list.into_iter().collect::<Vec<_>>();
    assert_eq!(vec[0], "nice");
    assert_eq!(vec[1], "test");
    assert_eq!(vec[2], "hallo");
    assert_eq!(vec.get(3), None);
}

#[test]
fn iter_mut() {
    let mut list = create_list(&[1, 2, 3]);
    let iter = list.iter_mut();
    iter.for_each(|item| {
        *item *= 2;
    });
    assert_eq!(list, create_list(&[2, 4, 6]));
}

#[test]
fn get_large_number() {
    let mut list = LinkedList::new();
    // i had to make this smaller because of miri
    for i in 0..10000 {
        list.push_front(i);
    }
    assert_eq!(list.get(9999), Some(&0));
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
fn list_len() {
    let list = create_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(list.len(), 9);
}

#[test]
fn std_traits() {
    let mut list1 = create_list(&[1, 5, 732, 533]);
    let list2 = create_list(&[1, 5, 732, 533]);
    assert_eq!(list1, list2);

    list1.extend([99, 100].iter().cloned());
    assert_eq!(list1, create_list(&[1, 5, 732, 533, 99, 100]));

    let vec1 = vec![1, 5, 732, 533, 99, 100];
    let list_from_vec = vec1.into_iter().collect::<LinkedList<_>>();
    assert_eq!(list1, list_from_vec);
}

#[test]
fn into_iter_not_consumed() {
    let list = create_list(&[1, 2, 4, 6, 7, 4, 5, 7, 57, 5]);
    list.into_iter();
}

/// Creates an owned list from a slice, not efficient at all but easy to use
fn create_list<T: Clone>(iter: &[T]) -> LinkedList<T> {
    iter.into_iter().cloned().collect()
}
