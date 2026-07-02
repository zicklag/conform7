# PLAN-12 Review Fixes

## Critical Issues to Fix

### Issue 1: NegatedNoncopularVerbPresent early length check
**File**: `crates/conform7-syntax/src/preform_internal.rs` (line ~1080)

**Problem**: `wording.len() < 3` guard rejects contracted forms like "doesn't carry" (2 words). Should be `wording.len() < 2`.

### Issue 2: set_verb_usage overloads VerbalCertainty annotation
**File**: `crates/conform7-syntax/src/parse_node.rs` (line ~177)

**Problem**: `set_verb_usage` stores the verb usage ref as a `VerbalCertainty` annotation, using a magic threshold (> 100) to distinguish verb usage refs from certainty levels. This is fragile.

**Fix**: Add dedicated `VerbUsage(VerbUsageRef)` and `PrepositionRef(PrepositionRef)` variants to the `Annotation` enum.

### Issue 3: set_preposition overloads LinguisticErrorHere annotation
**File**: `crates/conform7-syntax/src/parse_node.rs` (line ~199)

**Problem**: `set_preposition` stores the preposition ref as a `LinguisticErrorHere` annotation.

**Fix**: Use the new `PrepositionRef` annotation variant.

### Issue 4: set_second_preposition overloads SentenceIsExistential annotation
**File**: `crates/conform7-syntax/src/parse_node.rs` (line ~219)

**Problem**: `set_second_preposition` stores the second preposition ref as `SentenceIsExistential(true)`.

**Fix**: Add a `SecondPrepositionRef(PrepositionRef)` variant to the `Annotation` enum.

## Warnings to Fix

### Issue 5: Copular detection in default_verb
**File**: `crates/conform7-syntax/src/verb_phrases.rs` (line ~599)

**Fix**: Pass the verb ref through the call chain and check `verbs_registry.copular_verb == Some(verb_ref)`.

### Issue 6: Corrective surgery stubs
**File**: `crates/conform7-syntax/src/verb_phrases.rs` (lines ~680, ~712, ~739)

**Fix**: Implement the three corrective surgery functions or document them as deferred.

### Issue 7: build_diagram doesn't set certainty annotations
**File**: `crates/conform7-syntax/src/verb_phrases.rs` (line ~491)

**Fix**: Set `VerbalCertainty` annotation on the VERB_NT node.

### Issue 8: No tests for annotation variants
**File**: `crates/conform7-syntax/src/parse_node.rs` (line ~452)

**Fix**: Add unit tests for the new annotation variants and methods.
