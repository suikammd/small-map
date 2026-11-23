# Changelog

## 0.2.0 (unreleased)

### no_std and alloc support

SmallMap can now be built without std. Inline storage remains allocation-free;
spilling beyond the inline capacity uses `hashbrown::HashMap` and requires the
final application to provide a global allocator. This release also includes the
previously introduced SmallSet and the retain panic-safety fix (#7, #10).

### Breaking: default heap and inline hasher types

Normal std configuration is unchanged. When std is disabled, the default heap
hasher changes from `std::collections::hash_map::RandomState` to
`foldhash::fast::RandomState`. Without any external hasher feature, **the default
inline hasher also changes** from std's `RandomState` to foldhash's.

`SH` and `SI` are separate choices: this does not mean heap and inline storage
have swapped roles or that all hasher backends obtain std's security properties.
`ahash` and other hasher features can still supply their own builder types, while
`RandomState` from std remains a concrete type distinct from foldhash's.

### Migration

If you use Rust std, explicitly enable std (or retain it in the default features).
Existing code using std's `RandomState` as an explicit heap-hasher type continues
to work, including that constructor with `std` and `ahash` simultaneously.
Disabling std implies choosing the non-std default; do not translate Cargo using
`default-features = false` into an implicit std heap default.

For genuinely no_std targets, either use `small-map` with `default-features = false`
and accept the foldhash default, or choose a supported external hasher. Caller-side
`SH`/`SI` generic parameters and `with_hashers`/`with_capacity_and_hashers` allow
separate selection of heap and inline hashers.

On std-supported targets, code that explicitly wants std `RandomState` should
specify it as the heap hasher with a fully qualified type instead of relying on
feature-dependent inference. Library crate `std` availability and the concrete
heap-hasher type are distinct from optional hasher features.

The no_std default is not a substitute for std's HashDoS resistance. foldhash is
non-cryptographic and may seed differently across targets and atomic capabilities;
choose appropriate hashers for adversarial inputs and platform support.

### Serde and generic hashing

Serialization/deserialization now covers custom inline hashers and
`LINEAR_THRESHOLD`, including configurations such as `RapidSmallMap`. These are
existing SmallMap parameters, not newly added storage concepts.

### Compatibility checks

CI covers std `RandomState` preservation, foldhash-backed default construction,
serde inline and heap paths, and riscv32i nil/no_std builds subject to each
hasher's required platform capabilities. Optional dependencies retain their own
platform-specific requirements rather than providing a blanket support guarantee.
