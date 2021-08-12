use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use datastructures::linked_list::LinkedList;
use datastructures::packed_linked_list::PackedLinkedList;

fn create_random_list(size: usize) -> LinkedList<i32> {
    let mut number = 837582573;
    let mut list = LinkedList::new();
    for _ in 0..size {
        // just random stuff I cam up with, does not need to be actually random
        number = (number ^ (number << 5)) >> 3;
        list.push_back(number);
    }
    list
}

fn create_random_packed_list_16(size: usize) -> PackedLinkedList<i32, 16> {
    let mut number = 837582573;
    let mut list = PackedLinkedList::new();
    for _ in 0..size {
        // just random stuff I cam up with, does not need to be actually random
        number = (number ^ (number << 5)) >> 3;
        list.push_back(number);
    }
    list
}

fn create_random_packed_list_128(size: usize) -> PackedLinkedList<i32, 128> {
    let mut number = 837582573;
    let mut list = PackedLinkedList::new();
    for _ in 0..size {
        // just random stuff I cam up with, does not need to be actually random
        number = (number ^ (number << 5)) >> 3;
        list.push_back(number);
    }
    list
}

fn push_back(c: &mut Criterion) {
    let mut group = c.benchmark_group("push_back");
    for i in [100, 1_0000_00].iter() {
        group.bench_with_input(BenchmarkId::new("create_random_list", i), i, |b, i| {
            b.iter(|| create_random_list(*i))
        });
        group.bench_with_input(
            BenchmarkId::new("create_random_packed_list_16", i),
            i,
            |b, i| b.iter(|| create_random_packed_list_16(*i)),
        );
        group.bench_with_input(
            BenchmarkId::new("create_random_packed_list_128", i),
            i,
            |b, i| b.iter(|| create_random_packed_list_128(*i)),
        );
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = push_back
);
criterion_main!(benches);
