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
fn iter_single_node() {
    let mut list = PackedLinkedList::<_, 16>::new();
    list.push_front("2");
    list.push_front("1");
    let mut iter = list.iter();
    assert_eq!(iter.next(), Some(&"1"));
    assert_eq!(iter.next(), Some(&"2"));
    assert_eq!(iter.next(), None);
}

fn create_list<T: Clone, const COUNT: usize>(iter: &[T]) -> PackedLinkedList<T, COUNT> {
    //iter.into_iter().cloned().collect()
    todo!()
}
