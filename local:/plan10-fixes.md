# PLAN-10 Review Fixes

## Critical Issues to Fix

### Issue 1: Article payload never propagated from sub-nonterminal matches
**File**: `crates/conform7-syntax/src/preform.rs` (line ~792) and `crates/conform7-syntax/src/linguistics.rs` (line ~227)

**Problem**: `parse_np_articled` calls `match_nonterminal_impl(ctx, registry, "np-articled", wording)` which matches a regular (non-internal) NT. For regular NTs, `match_nonterminal_impl` always returns `Match { internal: None, ... }`. The code at line 232-239 checks `m.internal` for an `Article` payload, but it will always be `None`.

**Root cause**: `try_match_production` (preform.rs line ~919) calls `match_nonterminal_impl` for sub-NTs but discards the internal result. Internal payloads from sub-nonterminal matches (e.g., `<article>` inside `<np-articled>`) are not propagated up to the parent match.

**Fix approach**: The simplest fix is to have `parse_np_articled` re-parse the article from the matched wording after the production match succeeds, rather than relying on internal payload propagation. This avoids changing the matching engine's API.

Alternatively, propagate internal payloads through the matching engine by:
1. Adding an `internal_payload` field to `Match` (or using the existing `internal` field)
2. Having `try_match_production` collect internal results from sub-NT matches
3. Having `match_nonterminal_impl` propagate the first internal result from a sub-NT match

The re-parse approach is simpler and less invasive.

### Issue 2: ArticleUsage.word is always empty string
**File**: `crates/conform7-syntax/src/linguistics.rs` (line ~236)

**Problem**: The `word` field in `ArticleUsage` is always set to `String::new()` (empty string) when created via `parse_np_articled`. The article internal NT only returns the article name ("definite"/"indefinite"), not the matched word.

**Fix**: Either:
- Have the article internal NT also return the matched word
- Or have `parse_np_articled` extract the word from the wording after matching

### Issue 3: ArticleUsage.article is String instead of Article
**File**: `crates/conform7-syntax/src/linguistics.rs` (line ~46)

**Problem**: `ArticleUsage.article` is `String` instead of `Article` as specified in PLAN-10.md.

**Fix**: Change `ArticleUsage.article` to use the `Article` struct.

## Warnings to Fix

### Issue 4: Article struct is defined but unused
**File**: `crates/conform7-syntax/src/linguistics.rs` (line ~35)

**Fix**: Use `Article` in `ArticleUsage` as described above.

## Suggestions to Address

### Issue 5: No test for public parse_noun_phrase function
**File**: `crates/conform7-syntax/src/linguistics.rs` (line ~267)

**Fix**: Add tests for the public `parse_noun_phrase` function.

### Issue 6: No test for fallback behavior
**File**: `crates/conform7-syntax/src/linguistics.rs` (line ~267)

**Fix**: Add test that `parse_noun_phrase` with `<np-articled>` fails on "xyzzy" (no article) but falls through to `<np-unparsed>`.

### Issue 7: test_add_article_annotation doesn't verify annotation
**File**: `crates/conform7-syntax/src/linguistics.rs` (line ~636)

**Fix**: Add a getter for article annotations on ParseNode and verify it in the test.

### Issue 8: ArticleInternal uses HashMap directly instead of SmallWordSet
**File**: `crates/conform7-syntax/src/preform_internal.rs` (line ~519)

**Fix**: Use `SmallWordSet<String>` instead of `HashMap<String, String>`.

### Issue 9: Individual test functions lack C reference comments
**File**: `crates/conform7-syntax/src/linguistics.rs` (line ~306)

**Fix**: Add per-test C reference comments.
