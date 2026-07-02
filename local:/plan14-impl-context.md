# PLAN-14 Implementation Context

## Plan
Read `plans/PLAN-14.md` for the full plan with all 7 task groups and success criteria.

## Current Codebase State

### Project structure
- Workspace root: `/home/zicklag/git/zicklag/conform7/`
- Two existing crates: `conform7-syntax` and `conform7-inter`
- New crate to create: `conform7-semantics`

### Key patterns
- Each crate has `Cargo.toml` and `src/lib.rs`
- Tests are `#[cfg(test)] mod tests { ... }` at the bottom of each module
- C .w files are referenced in doc comments
- Use `once_cell::sync::Lazy` for global statics (or `std::sync::LazyLock` on recent Rust)

### Build & test
```bash
cd /home/zicklag/git/zicklag/conform7
cargo test  # 417 tests currently pass
cargo clippy --all-targets
```

## Implementation Order

1. Create `crates/conform7-semantics/` crate structure
2. Implement `KindConstructor` struct in `kind_constructors.rs`
3. Implement `Kind` struct in `kinds.rs`
4. Define familiar kinds and constructors in `familiar_kinds.rs`
5. Implement the kind lattice in `lattice.rs`
6. Add textual I/O (Display + FromStr)
7. Integration tests
