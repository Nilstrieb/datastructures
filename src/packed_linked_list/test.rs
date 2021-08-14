use super::*;

#[test]
fn empty_unit_list() {
    PackedLinkedList::<(), 0>::new();
}

#[test]
fn push_front_single_node() {
    let mut list = PackedLinkedList::<_, 16>::new();
    list.push_front("hallo");
}
#[test]
fn push_front_multiple_nodes_count_1() {
    let mut list = PackedLinkedList::<_, 1>::new();
    list.push_front("3");
    list.push_front("2");
    list.push_front("1");
}

#[test]
fn push_front_multiple_nodes_count_2() {
    let mut list = PackedLinkedList::<_, 2>::new();
    list.push_front("3");
    list.push_front("2");
    list.push_front("1");
}

#[test]
fn pop_front() {
    let mut list = create_sized_list::<_, 2>(&[1, 2, 3, 4]);
    assert_eq!(list.pop_front(), Some(1));
    assert_eq!(list.pop_front(), Some(2));
    assert_eq!(list.pop_front(), Some(3));
    assert_eq!(list.pop_front(), Some(4));
    assert_eq!(list.pop_front(), None);
    assert_eq!(list.pop_front(), None);
}

#[test]
fn iter_single_node() {
    let mut list = PackedLinkedList::<_, 16>::new();
    list.push_front("2");
    list.push_front("1");
    let mut iter = list.iter();
    assert_eq!(iter.next(), Some(&"1"));
    assert_eq!(iter.next(), Some(&"2"));
    assert_eq!(iter.next(), None);
}

#[test]
fn into_iter() {
    let mut iter = create_list(&[1, 2, 3]).into_iter();
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), None);
}

#[test]
fn iter_mut() {
    let mut list = create_list(&[1, 2, 3, 4]);
    let mut iter_mut = list.iter_mut();
    *iter_mut.next().unwrap() = 10;
    assert!([10, 2, 3, 4].iter().zip(list.iter()).all(|(a, b)| a == b));
}

#[test]
fn from_iter() {
    let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let list = vec
        .clone()
        .into_iter()
        .collect::<PackedLinkedList<_, 1024>>();
    let list_iter = list.iter();
    assert!(list_iter.zip(vec.iter()).all(|(a, b)| a == b));
}

#[test]
fn get_cursor() {
    let list = create_list(&[1, 2, 3, 4, 5, 6]);
    let mut cursor = list.cursor_front();
    assert_eq!(cursor.get(), Some(&1));
    cursor.move_next();
    assert_eq!(cursor.get(), Some(&2));
    cursor.move_prev();
    assert_eq!(cursor.get(), Some(&1));
    cursor.move_prev();
    assert_eq!(cursor.get(), None);
    cursor.move_prev();
    assert_eq!(cursor.get(), Some(&6));
    cursor.move_prev();
    cursor.move_prev();
    cursor.move_prev();
    cursor.move_prev();
    cursor.move_prev();
    assert_eq!(cursor.get(), Some(&1));
    cursor.move_prev();
    cursor.move_next();
    assert_eq!(cursor.get(), Some(&1));
}

#[test]
#[ignore]
fn insert_cursor() {
    let mut list = create_list(&[1, 2, 3, 4, 5, 6]);
    let mut cursor = list.cursor_mut_front();
    cursor.move_prev();
    cursor.move_prev();
    *cursor.get_mut().unwrap() = 100;
    cursor.move_next();
    cursor.move_next();
    cursor.insert_after(11);
    cursor.insert_before(0);
    cursor.move_next();
    assert_eq!(cursor.replace(12), Some(1));
    assert_eq!(cursor.get(), Some(&12));
    assert_eq!(cursor.remove(), Some(12));
    assert_eq!(list, create_list(&[0, 11, 2, 3, 4, 5, 100]));
}

#[test]
fn insert_after_cursor() {
    let mut list = create_sized_list(&[1, 2, 3]);
    let mut cursor = list.cursor_mut_front();
    // case 3
    cursor.insert_after(11);
    assert_eq!(list, create_list(&[1, 11, 2, 3]));

    let mut list = create_sized_list::<_, 4>(&[1, 2, 3, 4]);
    let mut cursor = list.cursor_mut_front();
    // case 4
    cursor.insert_after(11);
    assert_eq!(list, create_sized_list(&[1, 11, 2, 3, 4]));
}

fn create_list<T: Clone>(iter: &[T]) -> PackedLinkedList<T, 8> {
    iter.into_iter().cloned().collect()
}

fn create_sized_list<T: Clone, const COUNT: usize>(iter: &[T]) -> PackedLinkedList<T, COUNT> {
    iter.into_iter().cloned().collect()
}
