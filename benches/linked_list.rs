use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use datastructures::linked_list::LinkedList;

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

fn create_random_std_list(size: usize) -> std::collections::LinkedList<i32> {
    let mut number = 837582573;
    let mut list = std::collections::LinkedList::new();
    for _ in 0..size {
        // just random stuff I cam up with, does not need to be actually random
        number = (number ^ (number << 5)) >> 3;
        list.push_back(number);
    }
    list
}

fn list_length(list: &LinkedList<i32>) -> usize {
    list.len()
}

fn bench_list_length(c: &mut Criterion) {
    let list = create_random_list(100);
    c.bench_function("Short list length", |b| {
        b.iter(|| list_length(black_box(&list)))
    });
    let list = create_random_list(10_000_000);
    c.bench_function("Long list length", |b| {
        b.iter(|| list_length(black_box(&list)))
    });
}

fn push_back(c: &mut Criterion) {
    let mut group = c.benchmark_group("push_back");
    for i in [100, 10_000_000].iter() {
        group.bench_with_input(BenchmarkId::new("create_random_std_list", i), i, |b, i| {
            b.iter(|| create_random_std_list(*i))
        });
        group.bench_with_input(BenchmarkId::new("create_random_list", i), i, |b, i| {
            b.iter(|| create_random_list(*i))
        });
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = bench_list_length, push_back
);
criterion_main!(benches);
