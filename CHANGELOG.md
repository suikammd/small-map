# Changelog

## 0.2.0 (unreleased)

### no_std and alloc-only support

`small-map` can now be used with `default-features = false` without std:

- inline storage remains allocation-free;
- spilling beyond the inline capacity is allocation-backed and requires the application to provide a global allocator;
- explicit custom heap and inline hashers are supported on supported targets.

### Breaking: default heap-hasher type outside std

`std` is retained in the default features, and normal std usage keeps
`std::collections::hash_map::RandomState` as the default heap hasher.

When std is disabled, the default heap hasher changes to
`foldhash::fast::RandomState`, **a different concrete type from the standard
library's RandomState**. `AHasher` and other external hasher(s) do not imply the
std heap type; explicit SH/SI choices determine the actual hashers.

### Migration

- Keep `std` in dependencies (or explicitly request it) if your code relies on
  std's heap hasher and its HashDoS resistance. When migrating calls with
  explicit std `RandomState` constructor arguments, specify the heap-hasher type
  explicitly instead of relying on the implicit default heap type.
- For targets without std, use `small-map` with `default-features = false`, and
  either accept foldhash as the default hasher or select an external/custom
  hasher explicitly with `with_hashers` or `with_capacity_and_hashers`. Inline
  storage itself does not allocate; heap-spill paths require an allocator.
- `SmallSet<8, T, RandomState>::with_hasher(RandomState::new())` remains valid
  with std explicitly enabled. Omitting SH and leaving inference to decide may
  not be preserved across std-enabled versus std-disabled configurations.
- `ahash` does not imply non-std. A dependency can be built in no_std mode while
  an std-enabled application explicitly supplies the standard library's RandomState.
  Its dependencies may have their own requirements (e.g. pointer atomics).

See the README for practical no_std examples and the required heap-type
annotations when migrating older code.

### Compatibility notes

- The existing `SI` parameter is unchanged: inline hashers retain their existing
  rapidhash-first feature and type-selection priority.
- RISC-V `riscv32i-unknown-none-elf` is checked explicitly by CI. Optional
  `ahash` remains constrained by its dependency's pointer-atomic requirements
  and should not be used as evidence that all platforms or all hasher features
  are supported everywhere.
