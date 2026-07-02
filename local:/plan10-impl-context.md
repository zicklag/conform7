# PLAN-10 Implementation Context

## Plan
Read `plans/PLAN-10.md` for the full plan.

## Current Codebase State

### Project structure
- Workspace root: `/home/zicklag/git/zicklag/conform7/`
- Two crates: `conform7-syntax` and `conform7-inter`
- All work happens in `conform7-syntax` for this plan

### Key files to modify

1. **`crates/conform7-syntax/src/node_type.rs`** — Add linguistics node types (VERB_NT, UNPARSED_NOUN_NT, etc.)
2. **`crates/conform7-syntax/src/parse_node.rs`** — May need annotation additions
3. **`crates/conform7-syntax/src/preform.rs`** — May need minor API changes
4. **`crates/conform7-syntax/src/preform_internal.rs`** — Add article internal NTs
5. **`crates/conform7-syntax/src/lib.rs`** — Add new module exports
6. **New: `crates/conform7-syntax/src/linguistics.rs`** — Linguistics module

### Current patterns

**NodeType enum** (node_type.rs):
```rust
pub enum NodeType {
    Root,
    Heading,
    Sentence,
    Include,
    Table,
    Equation,
    Use,
    // ... more as needed
}
```

**ParseNode** (parse_node.rs):
```rust
pub struct ParseNode {
    node_type: NodeType,
    wording: Wording,
    down: Option<Box<ParseNode>>,
    next: Option<Box<ParseNode>>,
    next_alternative: Option<Box<ParseNode>>,
    annotation: Option<Annotation>,
}
```

**Annotation** (parse_node.rs):
```rust
pub enum Annotation {
    HeadingLevel(HeadingLevel),
    // Add ArticleUsage here
}
```

**InternalNonterminal trait** (preform.rs):
```rust
pub trait InternalNonterminal: Send + Sync {
    fn match_internal(
        &self,
        ctx: &PreformContext,
        wording: Wording,
    ) -> Option<InternalResult>;
}
```

**InternalRegistry** (preform.rs):
```rust
pub struct InternalRegistry {
    implementations: HashMap<String, Box<dyn InternalNonterminal>>,
}
impl InternalRegistry {
    pub fn basic() -> Self { ... }
    // Add: pub fn linguistics() -> Self { ... }
}
```

**PreformContext** (preform.rs):
```rust
pub struct PreformContext<'a> {
    pub grammar: &'a Grammar,
    pub word_text: &'a [&'a str],
    pub is_paragraph_start: bool,
}
```

**Wording** (wording.rs):
```rust
pub struct Wording {
    pub start: u32,
    pub end: u32,
}
```

### C Reference files
- `gitignore/inform/services/linguistics-module/Chapter 1/Diagrams.w` — node types
- `gitignore/inform/services/linguistics-module/Chapter 2/Articles.w` — article system
- `gitignore/inform/services/linguistics-module/Chapter 4/Noun Phrases.w` — noun phrase parsing
- `gitignore/inform/services/linguistics-module/Chapter 1/Stock Control.w` — linguistic stock
- `gitignore/inform/inform7/Internal/Languages/English/Syntax.preform` — real grammar

### Test patterns
Tests use `#[cfg(test)] mod tests { ... }` at the bottom of each module file.
Integration tests use doc-tests in the public API.

### Build & test
```bash
cd /home/zicklag/git/zicklag/conform7
cargo test  # 244 tests currently pass
cargo clippy --all-targets
```

## Implementation Order

1. Add linguistics node types to `NodeType` enum
2. Add `ArticleUsage` annotation to `Annotation` enum
3. Create `linguistics.rs` module with article system and noun phrase parsing
4. Add article internal NTs to `preform_internal.rs`
5. Add `linguistics()` constructor to `InternalRegistry`
6. Wire up in `lib.rs`
7. Add tests
8. Run `cargo test` and `cargo clippy --all-targets`
