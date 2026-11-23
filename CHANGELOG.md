# Changelog

## 0.2.0 (unreleased)

### no_std and alloc support

SmallMap can now be built without std using an indexmap-style explicit-hasher
configuration. Inline storage remains allocation-free; spilling beyond the
inline capacity uses `hashbrown::HashMap` and requires the final application to
provide a global allocator. This release also includes the previously introduced
SmallSet and the retain panic-safety fix (#7, #10).

### Breaking: explicit hashers without std

The default std configuration retains `std::collections::hash_map::RandomState`
as the heap hasher and preserves the existing inline-hasher selection order.
Other hasher features do not replace std's heap-hasher default.

With std disabled, `SmallMap` and `SmallSet` require explicit heap (`SH`) and
inline (`SI`) hasher types. There is no implicit std or foldhash fallback. Use a
supported external hasher's `SmallMap`/`SmallSet` alias, or supply builders with
`with_hashers`/`with_capacity_and_hashers`. Heap and inline storage remain
separate choices, and each selected hasher has its own security and target
requirements.

### Migration

If you use Rust std, retain the default features or explicitly enable `std` with
`default-features = false`. Existing std default declarations then retain their
usual concrete types. Code explicitly using std's `RandomState` can continue to
do so with that feature enabled.

For genuinely no_std targets, choose an external hasher supported by your target
and use its alias, or explicitly specify `SH` and `SI`. On std-capable targets
where small-map's `std` feature is disabled, callers using std's `RandomState`
must also spell out the inline-builder type, for example
`SmallSet<N, T, RandomState, RandomState>`.

Cargo resolves one small-map configuration per dependency graph. Enabling `std`
makes its usual defaults available throughout that graph; no_std callers must
therefore choose their hashers explicitly rather than relying on an implicit
feature-specific heap type.

### Serde and generic hashing

Serialization/deserialization now handles custom inline hashers and
`LINEAR_THRESHOLD`, including configurations such as `RapidSmallMap`. These are
existing SmallMap parameters, not newly added storage concepts. Tests cover both
inline and direct heap deserialization with a size-hint-capable input.

### Compatibility checks

CI covers std default construction, explicit-hasher no_std configurations,
serde inline and heap paths, and riscv32i no_std builds with supported external
hashers. Optional dependencies retain their own platform requirements.
