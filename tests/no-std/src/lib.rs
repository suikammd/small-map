#![no_std]

use core::hash::{BuildHasher, Hasher};

use small_map::{SmallMap, SmallSet};

pub struct TestHasher(u64);

impl Hasher for TestHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }
}

#[derive(Clone, Default)]
pub struct TestBuildHasher;

impl BuildHasher for TestBuildHasher {
    type Hasher = TestHasher;

    fn build_hasher(&self) -> Self::Hasher {
        TestHasher(0xcbf29ce484222325)
    }
}

pub type TestMap = SmallMap<2, u32, u32, TestBuildHasher, TestBuildHasher, 0>;
pub type TestSet = SmallSet<2, u32, TestBuildHasher, TestBuildHasher, 0>;

pub fn exercise_default_map_and_set() -> (SmallMap<2, u32, u32>, SmallSet<2, u32>) {
    let mut map = SmallMap::new();
    let mut set = SmallSet::new();
    for i in 0..4 {
        map.insert(i, i * 2);
        set.insert(i);
    }
    assert_eq!(map.get(&1), Some(&2));
    assert!(set.contains(&1));
    map.remove(&0);
    set.remove(&0);
    (map, set)
}

pub fn exercise_custom_map_and_set() -> (TestMap, TestSet) {
    let mut map = TestMap::with_hashers(TestBuildHasher, TestBuildHasher);
    let mut set = TestSet::with_hashers(TestBuildHasher, TestBuildHasher);
    for i in 0..4 {
        map.insert(i, i * 2);
        set.insert(i);
    }
    assert_eq!(map.get(&1), Some(&2));
    assert!(set.contains(&1));
    map.remove(&0);
    set.remove(&0);
    (map, set)
}

#[cfg(feature = "fxhash")]
pub fn exercise_fxhash() -> small_map::FxSmallMap<2, u32, u32> {
    let mut map = small_map::FxSmallMap::new();
    for i in 0..4 {
        map.insert(i, i * 2);
    }
    map
}

#[cfg(feature = "rapidhash")]
pub fn exercise_rapidhash() -> small_map::RapidSmallMap<2, u32, u32> {
    let mut map = small_map::RapidSmallMap::new();
    for i in 0..4 {
        map.insert(i, i * 2);
    }
    map
}

#[cfg(feature = "ahash")]
pub fn exercise_ahash() -> small_map::ASmallMap<2, u32, u32> {
    let mut map = small_map::ASmallMap::new();
    for i in 0..4 {
        map.insert(i, i * 2);
    }
    map
}

#[cfg(feature = "serde")]
pub fn assert_serde_impls() {
    fn assert_impl<T>()
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
    }

    assert_impl::<SmallMap<2, u32, u32>>();
    assert_impl::<TestMap>();
    #[cfg(feature = "fxhash")]
    assert_impl::<small_map::FxSmallMap<2, u32, u32>>();
    #[cfg(feature = "rapidhash")]
    assert_impl::<small_map::RapidSmallMap<2, u32, u32>>();
    #[cfg(feature = "ahash")]
    assert_impl::<small_map::ASmallMap<2, u32, u32>>();
}
