#![feature(fn_align)]

//! > NOTE: This crate is generally *slower* than using `Vec::binary_search` over a pre-sorted
//! > vector, contrary to the claims in the referenced paper, and is mainly presented for
//! > curiosity's sake at this point.
//!
//! This crate provides a data structure for approximate lookups in ordered collections.
//!
//! More concretely, given a set `A` of `n` values, and a query value `x`, this library provides an
//! efficient mechanism for finding the smallest value in `A` that is greater than or equal to `x`.
//! In particular, this library caters to the important case where there are many such queries to
//! the same array, `A`.
//!
//! This library is constructed from the best solution identified in [Array Layouts for
//! Comparison-Based Searching](https://arxiv.org/abs/1509.05053) by Paul-Virak Khuong and Pat
//! Morin. For more information, see the paper, [their
//! website](http://cglab.ca/~morin/misc/arraylayout-v2/), and the [C++ implementation
//! repository](https://github.com/patmorin/arraylayout).
//!
//! # Current implementation
//!
//! At the time of writing, this implementation uses a branch-free search over an
//! Eytzinger-arranged array with masked prefetching based on the [C++
//! implementation](https://github.com/patmorin/arraylayout/blob/3f20174a2a0ab52c6f37f2ea87d087307f19b5ee/src/eytzinger_array.h#L253)
//! written by the authors of the aforementioned paper. This is the recommended algorithm from the
//! paper, and what the authors suggested in
//! https://github.com/patmorin/arraylayout/issues/3#issuecomment-338472755.
//!
//! Note that prefetching is *only* enabled with the (non-default) `nightly` feature due to
//! https://github.com/aweinstock314/prefetch/issues/1. Suggestions for workarounds welcome.
//!
//! # Performance
//!
//! The included benchmarks can be run with
//!
//! ```console,ignore
//! $ cargo +nightly bench --features nightly
//! ```
//!
//! This will benchmark both construction and search with different number of values, and
//! differently sized values -- look for the line that aligns closest with your data. The general
//! trend is that `ordsearch` is faster when `n` is smaller and `T` is larger as long as you
//! compile with
//! [`target-cpu=native`](https://github.com/jonhoo/ordsearch/issues/2#issuecomment-390441137) and
//! [`lto=thin`](https://github.com/jonhoo/ordsearch/issues/2#issuecomment-390446671). The
//! performance gain seems to be best on Intel processors, and is smaller since the (relatively)
//! recent improvement to [SliceExt::binary_search
//! performance](https://github.com/rust-lang/rust/pull/45333).
//!
//! Below are [summarized](https://github.com/BurntSushi/cargo-benchcmp) results from an AMD
//! ThreadRipper 2600X CPU run with:
//!
//! ```console
//! $ rustc +nightly --version
//! rustc 1.28.0-nightly (e3bf634e0 2018-06-28)
//! $ env CARGO_INCREMENTAL=0 RUSTFLAGS='-C target-cpu=native -C lto=thin' cargo +nightly bench --features nightly
//! ```
//!
//! Compared to binary search over a sorted vector:
//!
//! ```diff,ignore
//!  name           sorted_vec ns/iter  this ns/iter  diff ns/iter   diff %  speedup
//! -u32::l1        49                  54                       5   10.20%   x 0.91
//! +u32::l1_dup    40                  35                      -5  -12.50%   x 1.14
//! -u32::l2        63                  72                       9   14.29%   x 0.88
//! +u32::l2_dup    64                  62                      -2   -3.12%   x 1.03
//! -u32::l3        120                 273                    153  127.50%   x 0.44
//! -u32::l3_dup    117                 219                    102   87.18%   x 0.53
//! +u8::l1         42                  37                      -5  -11.90%   x 1.14
//! +u8::l1_dup     29                  28                      -1   -3.45%   x 1.04
//! +u8::l2         43                  49                       6   13.95%   x 0.88
//! -u8::l2_dup     33                  35                       2    6.06%   x 0.94
//! -u8::l3         45                  66                      21   46.67%   x 0.68
//! -u8::l3_dup     35                  51                      16   45.71%   x 0.69
//! -usize::l1      49                  54                       5   10.20%   x 0.91
//! +usize::l1_dup  38                  37                      -1   -2.63%   x 1.03
//! -usize::l2      65                  76                      11   16.92%   x 0.86
//! +usize::l2_dup  65                  64                      -1   -1.54%   x 1.02
//! -usize::l3      141                 303                    162  114.89%   x 0.47
//! -usize::l3_dup  140                 274                    134   95.71%   x 0.51
//! ```
//!
//! Compared to a `BTreeSet`:
//!
//! ```diff,ignore
//!  name           btreeset ns/iter  this ns/iter  diff ns/iter   diff %  speedup
//! +u32::l1        68                54                     -14  -20.59%   x 1.26
//! +u32::l1_dup    45                35                     -10  -22.22%   x 1.29
//! +u32::l2        88                72                     -16  -18.18%   x 1.22
//! -u32::l2_dup    61                62                       1    1.64%   x 0.98
//! +u32::l3        346               273                    -73  -21.10%   x 1.27
//! -u32::l3_dup    136               219                     83   61.03%   x 0.62
//! +u8::l1         45                37                      -8  -17.78%   x 1.22
//! +u8::l1_dup     31                28                      -3   -9.68%   x 1.11
//! -u8::l2         44                49                       5   11.36%   x 0.90
//! -u8::l2_dup     31                35                       4   12.90%   x 0.89
//! -u8::l3         43                66                      23   53.49%   x 0.65
//! -u8::l3_dup     30                51                      21   70.00%   x 0.59
//! +usize::l1      67                54                     -13  -19.40%   x 1.24
//! +usize::l1_dup  44                37                      -7  -15.91%   x 1.19
//! +usize::l2      89                76                     -13  -14.61%   x 1.17
//! -usize::l2_dup  60                64                       4    6.67%   x 0.94
//! +usize::l3      393               303                    -90  -22.90%   x 1.30
//! -usize::l3_dup  163               274                    111   68.10%   x 0.59
//! ```
//!
//! # Future work
//!
//!  - [ ] Implement aligned operation: https://github.com/patmorin/arraylayout/blob/3f20174a2a0ab52c6f37f2ea87d087307f19b5ee/src/eytzinger_array.h#L204
//!  - [ ] Implement deep prefetching for large `T`: https://github.com/patmorin/arraylayout/blob/3f20174a2a0ab52c6f37f2ea87d087307f19b5ee/src/eytzinger_array.h#L128
//!
#![deny(missing_docs)]
#![no_std]
#![cfg_attr(feature = "nightly", feature(concat_idents))]
#![cfg_attr(feature = "nightly", feature(core_intrinsics))]

extern crate alloc;
#[cfg(test)]
extern crate std;

use alloc::vec::Vec;
use core::{
    borrow::{Borrow, BorrowMut},
    mem::{self, size_of},
};

/// A collection of ordered items that can efficiently satisfy queries for nearby elements.
///
/// The most interesting method here is `find_gte`.
///
/// # Examples
///
/// ```
/// # use ordsearch::OrderedCollection;
/// let x = OrderedCollection::from(vec![1, 2, 4, 8, 16, 32, 64]);
/// assert_eq!(x.find_gte(0), Some(&1));
/// assert_eq!(x.find_gte(1), Some(&1));
/// assert_eq!(x.find_gte(3), Some(&4));
/// assert_eq!(x.find_gte(6), Some(&8));
/// assert_eq!(x.find_gte(8), Some(&8));
/// assert_eq!(x.find_gte(64), Some(&64));
/// assert_eq!(x.find_gte(65), None);
/// ```
// #[repr(align(64))]
pub struct OrderedCollection<T> {
    // find_gte_fn: FindGteFn<T>,
    items: Vec<T>,
}

impl<T: Ord> From<Vec<T>> for OrderedCollection<T> {
    /// Construct a new `OrderedCollection` from a vector of elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ordsearch::OrderedCollection;
    /// let a = OrderedCollection::from(vec![42, 89, 7, 12]);
    /// assert_eq!(a.find_gte(50), Some(&89));
    /// ```
    fn from(mut v: Vec<T>) -> OrderedCollection<T> {
        v.sort_unstable();
        Self::from_sorted_iter(v)
    }
}

/// Insert items from the sorted iterator `I` into `Vec<T>` in complete binary tree order.
///
/// Requires `I` to be a sorted iterator.
/// Requires `Vec<T>` capacity to be set to the number of elements in `iter`.
/// The length of `Vec<T>` will not be changed by this function.
fn eytzinger_walk<I, T>(context: &mut (Vec<T>, I), i: usize)
where
    I: Iterator<Item = T>,
{
    let (v, _) = context;
    if i >= v.capacity() {
        return;
    }

    // visit left child
    eytzinger_walk(context, 2 * i + 1);

    // reborrow context
    let (v, iter) = context;

    // put data at the root
    // we know the pointer arithmetics below is safe because we set the Vec's capacity to
    // the length of the iterator.
    let value = iter.next().unwrap();
    unsafe {
        v.as_mut_ptr().add(i).write(value);
    }

    // visit right child
    eytzinger_walk(context, 2 * i + 2);
}

impl<T: Ord> OrderedCollection<T> {
    // this computation is a little finicky, so let's walk through it.
    //
    // we want to prefetch a couple of levels down in the tree from where we are.
    // however, we can only fetch one cacheline at a time (assume a line holds 64b).
    // we therefore need to find at what depth a single prefetch fetches all the descendants.
    // it turns out that, at depth k under some node with index i, the leftmost child is at:
    //
    //   2^k * i + 2^(k-1) + 2^(k-2) + ... + 2^0 = 2^k * i + 2^k - 1
    //
    // this follows from the fact that the leftmost immediate child of node i is at 2i + 1 by
    // recursively expanding i. if you're curious, the rightmost child is at:
    //
    //   2^k * i + 2^k + 2^(k-1) + ... + 2^1 = 2^k * i + 2^(k+1) - 1
    //
    // at depth k, there are 2^k children. we can fit 64/sizeof(T) children in a cacheline, so
    // we want to use the depth k that has 64/sizeof(T) children. so, we want:
    //
    //   2^k = 64/sizeof(T)
    //
    // but, we don't actually *need* k. we only ever use 2^k. so, we can just use 64/sizeof(T)
    // directly! nice. we call this the multiplier (because it's what we'll multiply i by).
    const MULTIPLIER: usize = 64 / core::mem::size_of::<T>();

    // now for those additions we had to do above. well, we know that the offset is really just
    // 2^k - 1, and we know that multiplier == 2^k, so we're done. right?
    //
    // right?
    //
    // well, only sort of. the prefetch instruction fetches the cache-line that *holds* the
    // given memory address. let's denote cache lines with []. what if we have:
    //
    //   [..., 2^k + 2^k-1] [2^k + 2^k, ...]
    //
    // essentially, we got unlucky with the alignment so that the leftmost child is not sharing
    // a cacheline with any of the other items at that level! that's not great. so, instead, we
    // prefetch the address that is half-way through the set of children. that way, we ensure
    // that we prefetch at least half of the items.
    const OFFSET: usize = Self::MULTIPLIER + Self::MULTIPLIER / 2;

    /// Construct a new `OrderedCollection` from an iterator over sorted elements.
    ///
    /// Note that if the iterator is *not* sorted, no error will be given, but lookups will give
    /// incorrect results. The given iterator must also implement `ExactSizeIterator` so that we
    /// know the size of the lookup array.
    ///
    /// # Examples
    ///
    /// Using an already-sorted iterator:
    ///
    /// ```
    /// # use std::collections::BTreeSet;
    /// # use ordsearch::OrderedCollection;
    ///
    /// let mut s = BTreeSet::new();
    /// s.insert(42);
    /// s.insert(89);
    /// s.insert(7);
    /// s.insert(12);
    /// let a = OrderedCollection::from_sorted_iter(s);
    /// assert_eq!(a.find_gte(50), Some(&89));
    /// ```
    ///
    /// Sorting a collection and then iterating (in this case, you'd likely use `new` instead):
    ///
    /// ```
    /// # use ordsearch::OrderedCollection;
    /// let mut v = vec![42, 89, 7, 12];
    /// v.sort_unstable();
    /// let a = OrderedCollection::from_sorted_iter(v);
    /// assert_eq!(a.find_gte(50), Some(&89));
    /// ```
    ///
    /// The `OrderedCollection` can also be over references to somewhere else:
    ///
    /// ```
    /// # use std::collections::BTreeSet;
    /// # use ordsearch::OrderedCollection;
    ///
    /// let mut s = BTreeSet::new();
    /// s.insert(42);
    /// s.insert(89);
    /// s.insert(7);
    /// s.insert(12);
    /// let a = OrderedCollection::from_sorted_iter(s.into_iter());
    /// assert_eq!(a.find_gte(50), Some(&89));
    /// ```
    ///
    pub fn from_sorted_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator<Item = T>,
    {
        let iter = iter.into_iter();
        let n = iter.len();
        let mut context = (Vec::with_capacity(n), iter);
        eytzinger_walk(&mut context, 0);
        let (mut items, _) = context;

        // it's now safe to set the length, since all `n` elements have been inserted.
        unsafe { items.set_len(n) };

        OrderedCollection {
            items,
            // find_gte_fn: choose_find_gte_fn::<T>(n),
        }
    }

    /// Construct a new `OrderedCollection` from a slice of elements.
    ///
    /// Note that the underlying slice will be reordered!
    ///
    /// # Examples
    ///
    /// ```
    /// # use ordsearch::OrderedCollection;
    /// let mut vals = [42, 89, 7, 12];
    /// let a = OrderedCollection::from_slice(&mut vals);
    /// assert_eq!(a.find_gte(&50), Some(&&89));
    /// ```
    pub fn from_slice(v: &mut [T]) -> OrderedCollection<&T> {
        v.sort_unstable();
        OrderedCollection::from_sorted_iter(v.iter())
    }

    /// Find the smallest value `v` such that `v >= x`.
    ///
    /// Returns `None` if there is no such `v`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ordsearch::OrderedCollection;
    /// let x = OrderedCollection::from(vec![1, 2, 4, 8, 16, 32, 64]);
    /// assert_eq!(x.find_gte(0), Some(&1));
    /// assert_eq!(x.find_gte(1), Some(&1));
    /// assert_eq!(x.find_gte(3), Some(&4));
    /// assert_eq!(x.find_gte(6), Some(&8));
    /// assert_eq!(x.find_gte(8), Some(&8));
    /// assert_eq!(x.find_gte(64), Some(&64));
    /// assert_eq!(x.find_gte(65), None);
    /// ```
    pub fn find_gte<X>(&self, x: X) -> Option<&T>
    where
        X: Ord + Borrow<T>,
    {
        // (self.find_gte_fn)(self, x.borrow())
        // if self.items.len() >= 32 * 1024 {
        //     find_gte::<T, true, true>(self, x.borrow())
        // } else {
        //     find_gte::<T, false, true>(self, x.borrow())
        // }

        find_gte::<_, true, true>(self, x.borrow())

        // choose_find_gte_fn(self.items.len())(self, x.borrow())
    }
}

type FindGteFn<T> = for<'a> fn(&'a OrderedCollection<T>, &T) -> Option<&'a T>;

fn choose_find_gte_fn<T: Ord>(n: usize) -> FindGteFn<T> {
    let prefetch = n >= 1024 * 32;
    // let branchless = n > 32 && n < 10 * 1024 * 1024;
    let prefetch = true;
    let branchless = true;
    match (prefetch, branchless) {
        (true, true) => find_gte::<T, true, true>,
        (true, false) => find_gte::<T, true, false>,
        (false, true) => find_gte::<T, false, true>,
        (false, false) => find_gte::<T, false, false>,
    }
}

/// This is a free-standing function that is used to implement `OrderedCollection::find_gte`.
///
/// This is done so that we can have specialized versions of the search algorithm for collections of different sizes.
// #[repr(align(32))]
// #[inline(never)]
fn find_gte<'a, T, const PREFETCH: bool, const BRANCHLESS: bool>(
    collection: &'a OrderedCollection<T>,
    x: &T,
) -> Option<&'a T>
where
    T: Ord,
{
    let mut i = 0;

    let prefetching_enabled = collection.items.len() >= 32768;
    let mask = prefetch_mask(collection.items.len()); // * usize::from(prefetching_enabled);

    let level: usize = 64;
    let mut mul = 2usize.pow(level as u32);
    let mul_off = 2usize.pow(level as u32) - 1;
    let cachline_off = 64 / 2 / mem::size_of::<T>();

    while i < collection.items.len() {
        if PREFETCH {
            let offset = (mul * i + mul_off + cachline_off).min(collection.items.len() - 1);
            // assert!(offset == 0, "{}", offset);
            // let offset =
            //     (OrderedCollection::<T>::MULTIPLIER * i + OrderedCollection::<T>::OFFSET) & mask;
            do_prefetch(collection.items.as_ptr().wrapping_add(offset));
        }

        // safe because i < collection.items.len()
        let value = unsafe { collection.items.get_unchecked(i) };

        if !BRANCHLESS {
            if value == x {
                return Some(value);
            }
        }
        // using branchless index update. At the moment compiler cannot reliably tranform
        // if expressions to branchless instructions like `cmov` and `setb`
        i = 2 * i + 1 + usize::from(x > value);
    }

    // we want ffs(~(i + 1))
    // since ctz(x) = ffs(x) - 1
    // we use ctz(~(i + 1)) + 1
    let j = (i + 1) >> ((!(i + 1)).trailing_zeros() + 1);
    if j == 0 {
        None
    } else {
        Some(unsafe { collection.items.get_unchecked(j - 1) })
    }
}

#[cfg(feature = "nightly")]
#[inline(always)]
fn do_prefetch<T>(addr: *const T) {
    unsafe {
        core::intrinsics::prefetch_read_data(addr, 3);
    }
}

#[cfg(not(feature = "nightly"))]
fn do_prefetch<T>(_addr: *const T) {}

/// Calculates prefetch mask for a given collection size.
///
/// Creates binary mask that fully covers given [`usize`] value (eg. for the value `0b100` the mask is `0b111`).
/// Prefetch mask is used to keep an element address inside an array boundaries when prefetching next values from the
/// memory. Although, it is totally valid to prefetch invalid addresses from x86 point of view[^1],
/// on some CPUs it can lead to performance degradation[^2].
///
/// [^1]: [Intel® 64 and IA-32 Architectures Software Developer’s Manual](https://software.intel.com/en-us/download/intel-64-and-ia-32-architectures-sdm-combined-volumes-1-2a-2b-2c-2d-3a-3b-3c-3d-and-4)
///
/// [^2]: [Array Layouts for Comparison-Based Searching. Section 5.3](https://arxiv.org/pdf/1509.05053.pdf)
fn prefetch_mask(n: usize) -> usize {
    if n > 0 {
        usize::max_value() >> n.leading_zeros()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn complete_exact() {
        let x = OrderedCollection::from(vec![1, 2, 4, 8, 16, 32, 64]);
        assert_eq!(x.find_gte(1), Some(&1));
        assert_eq!(x.find_gte(2), Some(&2));
        assert_eq!(x.find_gte(4), Some(&4));
        assert_eq!(x.find_gte(8), Some(&8));
        assert_eq!(x.find_gte(16), Some(&16));
        assert_eq!(x.find_gte(32), Some(&32));
        assert_eq!(x.find_gte(64), Some(&64));
    }

    #[test]
    fn complete_approximate() {
        let x = OrderedCollection::from(vec![1, 2, 4, 8, 16, 32, 64]);
        assert_eq!(x.find_gte(0), Some(&1));
        assert_eq!(x.find_gte(3), Some(&4));
        assert_eq!(x.find_gte(5), Some(&8));
        assert_eq!(x.find_gte(6), Some(&8));
        assert_eq!(x.find_gte(7), Some(&8));
        for i in 9..16 {
            assert_eq!(x.find_gte(i), Some(&16));
        }
        for i in 17..32 {
            assert_eq!(x.find_gte(i), Some(&32));
        }
        for i in 33..64 {
            assert_eq!(x.find_gte(i), Some(&64));
        }
        assert_eq!(x.find_gte(65), None);
    }

    #[test]
    fn unbalanced_exact() {
        let x = OrderedCollection::from(vec![1, 2, 4, 8, 16, 32, 64, 128, 256]);
        assert_eq!(x.find_gte(1), Some(&1));
        assert_eq!(x.find_gte(2), Some(&2));
        assert_eq!(x.find_gte(4), Some(&4));
        assert_eq!(x.find_gte(8), Some(&8));
        assert_eq!(x.find_gte(16), Some(&16));
        assert_eq!(x.find_gte(32), Some(&32));
        assert_eq!(x.find_gte(64), Some(&64));
        assert_eq!(x.find_gte(128), Some(&128));
        assert_eq!(x.find_gte(256), Some(&256));
    }

    #[test]
    fn unbalanced_approximate() {
        let x = OrderedCollection::from(vec![1, 2, 4, 8, 16, 32, 64, 128, 256]);
        assert_eq!(x.find_gte(0), Some(&1));
        assert_eq!(x.find_gte(3), Some(&4));
        assert_eq!(x.find_gte(5), Some(&8));
        assert_eq!(x.find_gte(6), Some(&8));
        assert_eq!(x.find_gte(7), Some(&8));
        for i in 9..16 {
            assert_eq!(x.find_gte(i), Some(&16));
        }
        for i in 17..32 {
            assert_eq!(x.find_gte(i), Some(&32));
        }
        for i in 33..64 {
            assert_eq!(x.find_gte(i), Some(&64));
        }
        for i in 65..128 {
            assert_eq!(x.find_gte(i), Some(&128));
        }
        for i in 129..256 {
            assert_eq!(x.find_gte(i), Some(&256));
        }
        assert_eq!(x.find_gte(257), None);
    }

    #[test]
    fn check_mask() {
        assert_eq!(prefetch_mask(0), 0b000);
        assert_eq!(prefetch_mask(1), 0b001);
        assert_eq!(prefetch_mask(2), 0b011);
        assert_eq!(prefetch_mask(3), 0b011);
        assert_eq!(prefetch_mask(4), 0b111);
        assert_eq!(prefetch_mask(usize::max_value()), usize::max_value());
    }
}
