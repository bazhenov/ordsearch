extern crate ordsearch;
extern crate rand;
extern crate rust_pairwise_testing;

use ordsearch::OrderedCollection;
use rand::{rngs::ThreadRng, thread_rng, Rng, RngCore};
use rust_pairwise_testing::{measure, report, Generator};
use std::{rc::Rc, time::Instant};

struct PayloadGenerator {
    collection: Rc<OrderedCollection<u32>>,
    rand: ThreadRng,
    size: usize,
    iteration: usize,
}

impl PayloadGenerator {
    fn new(size: usize) -> Self {
        let mut rand = thread_rng();
        let collection = Rc::new(Self::generate_new_collection(&mut rand, size));
        Self {
            collection,
            rand,
            size,
            iteration: 0,
        }
    }

    fn generate_new_collection(rng: &mut ThreadRng, size: usize) -> OrderedCollection<u32> {
        let mut v = vec![0; size];
        rng.fill(&mut v[..]);
        OrderedCollection::from(v)
    }
}

impl Generator for PayloadGenerator {
    type Output = (u32, Rc<OrderedCollection<u32>>);

    fn next_payload(&mut self) -> Self::Output {
        self.iteration += 1;
        if self.iteration % 1000 == 0 {
            self.collection = Rc::new(Self::generate_new_collection(&mut self.rand, self.size));
        }
        (self.rand.next_u32(), Rc::clone(&self.collection))
    }
}

fn main() {
    let mut gen_1_m = PayloadGenerator::new(1_000_000);
    let mut gen_100_k = PayloadGenerator::new(100_000);
    let mut gen_10_k = PayloadGenerator::new(10_000);
    let mut gen_1_k = PayloadGenerator::new(1_000);
    let iter = 50000;

    println!(
        "{:40} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "name", "B min", "C min", "min ∆", "B mean", "C mean", "mean ∆", "mean ∆ (%)"
    );

    let data = measure(&mut gen_1_k, ord_search, ord_search_2, iter);
    report("1K find_gte / find_gte_2", data, None).unwrap();

    let data = measure(&mut gen_10_k, ord_search, ord_search_2, iter);
    report("10K find_gte / find_gte_2", data, None).unwrap();

    let data = measure(&mut gen_100_k, ord_search, ord_search_2, iter);
    report("100K find_gte / find_gte_2", data, None).unwrap();

    let data = measure(&mut gen_1_m, ord_search, ord_search_2, iter);
    report("1M find_gte / find_gte_2", data, None).unwrap();
}

fn ord_search(input: &(u32, Rc<OrderedCollection<u32>>)) -> (u128, u32) {
    let (query, collection) = input;

    let start = Instant::now();
    let result = *collection.find_gte(*query).unwrap_or(&0);
    (start.elapsed().as_nanos(), result)
}

fn ord_search_2(input: &(u32, Rc<OrderedCollection<u32>>)) -> (u128, u32) {
    let (query, collection) = input;

    let start = Instant::now();
    let result = *collection.find_gte2(*query).unwrap_or(&0);
    (start.elapsed().as_nanos(), result)
}
