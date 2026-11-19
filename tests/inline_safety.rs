//! Targeted regression tests for inline-storage safety issues.
//!
//! These exercise the SIMD search path (`N >= LINEAR_THRESHOLD` and `len >=
//! MIN_LINEAR_LEN`) through the public API only, covering three classes of UB:
//!
//! The original revision's insertion failure can mask the later failures during
//! test setup, so these are not independent first-failure demonstrations for
//! that revision. Keeping any of the fixes reverted after making setup safe
//! still exposes the corresponding failure below.
//!
//! * `ctrl_write_on_simd_insert`: `AlignedGroups::ctrl` derived a `*mut` from `&self` and
//!   `insert_in_slot` wrote through it (SharedReadOnly write).
//! * `bucket_write_on_simd_paths`: `find(&self)` handed out buckets whose pointers were derived
//!   from shared references; `get_mut`, insert-update, and `remove_entry` then wrote through them.
//! * `tail_group_load_reads_uninit`: the tail `Group::load` in `find` reads a full `Group::WIDTH`
//!   window; bytes past `len` were never initialized (and for `len == N` with `N % WIDTH != 0` the
//!   window even left the array).

use std::hash::{BuildHasher, Hasher};

use small_map::SmallMap;

#[derive(Clone, Default)]
struct ZeroBuildHasher;

#[derive(Default)]
struct ZeroHasher;

impl BuildHasher for ZeroBuildHasher {
    type Hasher = ZeroHasher;

    fn build_hasher(&self) -> Self::Hasher {
        ZeroHasher
    }
}

impl Hasher for ZeroHasher {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, _bytes: &[u8]) {}
}

// Force SIMD lookup even when N is smaller than Group::WIDTH. A zero hash also
// makes every live control byte equal to the zeroed padding, so an incorrect
// tail mask would compare against uninitialized/outdated data under Miri.
type ForcedSimdMap<const N: usize> = SmallMap<N, i32, i32, ZeroBuildHasher, ZeroBuildHasher, 2>;

/// Inserting into an inline map that uses the SIMD path must be allowed to
/// write control bytes.
#[test]
fn ctrl_write_on_simd_insert() {
    let mut map: SmallMap<32, i32, i32> = SmallMap::new();
    for i in 0..18 {
        map.insert(i, i * 10);
    }
    assert_eq!(map.len(), 18);
    for i in 0..18 {
        assert_eq!(map.get(&i), Some(&(i * 10)));
    }
}

/// `get_mut`, insert-update, and `remove_entry` on the SIMD path must write
/// through pointers derived from `&mut self`, not `&self`.
#[test]
fn bucket_write_on_simd_paths() {
    let mut map: SmallMap<32, i32, i32> = SmallMap::new();
    for i in 0..18 {
        map.insert(i, i * 10);
    }

    // find() hit -> bucket.as_mut()
    *map.get_mut(&7).unwrap() = 777;
    assert_eq!(map.get(&7), Some(&777));

    // find() hit -> mem::replace(&mut bucket.as_mut().1, ...)
    assert_eq!(map.insert(9, 999), Some(90));
    assert_eq!(map.get(&9), Some(&999));

    // find() hit -> bucket.read() + erase()
    assert_eq!(map.remove_entry(&3), Some((3, 30)));
    assert_eq!(map.len(), 17);
    assert_eq!(map.get(&3), None);
    // Remaining entries are still reachable after swap-delete.
    for i in (0..18).filter(|i| *i != 3) {
        assert!(map.get(&i).is_some(), "missing key {i}");
    }
}

/// A `Group::load` that runs off the initialized control bytes must still only
/// read initialized, in-bounds memory. `17 % Group::WIDTH == 1` for every Group
/// implementation here (WIDTH 4, 8, or 16), so the load starts at `ctrl[16]`
/// and reads full WIDTH bytes regardless of how many complete groups precede it.
#[test]
fn tail_group_load_reads_uninit() {
    // Padding bytes inside the control array: N = 32, len = 17.
    let mut map: SmallMap<32, i32, i32> = SmallMap::new();
    for i in 0..17 {
        map.insert(i, i);
    }
    // Force a SIMD `find` that walks the tail group (hit and miss).
    assert_eq!(map.get(&16), Some(&16));
    assert_eq!(map.get(&1234), None);

    // Same, but at `len == N` with `N % Group::WIDTH != 0`: the tail window
    // used to extend past the end of `groups`.
    let mut map: SmallMap<17, i32, i32> = SmallMap::new();
    for i in 0..17 {
        map.insert(i, i);
    }
    assert_eq!(map.get(&16), Some(&16));
    assert_eq!(map.get(&1234), None);
}

fn exercise_clone_remove_reinsert<const N: usize>() {
    let mut map = ForcedSimdMap::<N>::new();
    for i in 0..N as i32 {
        map.insert(i, i * 10);
    }

    // The clone must initialize its own padding and tail before SIMD lookup.
    let mut cloned = map.clone();
    let last = N as i32 - 1;
    assert_eq!(cloned.get(&last), Some(&(last * 10)));
    assert_eq!(cloned.get(&-1), None);

    // Removing from the front exercises bucket read followed by swap-delete.
    // Reinsertion must overwrite stale ctrl data, and the returned bucket must
    // remain writable after crossing insert_unique_unchecked's call boundary.
    assert_eq!(cloned.remove_entry(&0), Some((0, 0)));
    let new_key = N as i32;
    let (key, value) = unsafe { cloned.insert_unique_unchecked(new_key, new_key * 10) };
    assert_eq!(*key, new_key);
    *value += 1;

    assert_eq!(cloned.get(&0), None);
    assert_eq!(cloned.get(&new_key), Some(&(new_key * 10 + 1)));
    for i in 1..N as i32 {
        assert_eq!(cloned.get(&i), Some(&(i * 10)));
    }
    assert_eq!(cloned.get(&-1), None);
}

#[test]
fn small_capacity_and_lifecycle_tail_safety() {
    // N=1 remains on the linear path. N=2 and N=17 both force a tail load on
    // every supported Group::WIDTH; N=2 is smaller than the group and N=17
    // reaches the farthest possible byte (one before groups + tail's end).
    exercise_clone_remove_reinsert::<1>();
    exercise_clone_remove_reinsert::<2>();
    exercise_clone_remove_reinsert::<17>();
}

#[test]
fn zst_bucket_mutation_and_removal() {
    let mut map: SmallMap<16, (), ()> = SmallMap::new();
    map.insert((), ());
    *map.get_mut(&()).unwrap() = ();
    assert_eq!(map.insert((), ()), Some(()));

    let mut cloned = map.clone();
    assert_eq!(cloned.remove_entry(&()), Some(((), ())));
    let (_key, value) = unsafe { cloned.insert_unique_unchecked((), ()) };
    *value = ();
    assert!(cloned.contains_key(&()));
}
