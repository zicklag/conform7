//! Conform7 Debug REPL — explore how the compiler interprets Inform 7 source.
//!
//! Usage:
//!   cargo run --bin conform7-syntax              # interactive REPL
//!   cargo run --bin conform7-syntax -- file.i7    # parse a file
//!   echo "The Lab is a room." | cargo run --bin conform7-syntax
//!
//! Each sentence is run through the full frontend pipeline:
//!   Lexer → Sentence Breaker → Preform Grammar → Verb Phrase Parsing
//!
//! The verb registry includes common English verbs (to be, to have, to carry,
//! to take, to put, to look, to say, to wear, to try) so most simple
//! Inform 7 sentences will parse.

use conform7_syntax::lexer::Lexer;
use conform7_syntax::parse_node::Annotation;
use conform7_syntax::preform::{
    match_nonterminal_impl, parse_preform_grammar, InternalPayload, InternalRegistry,
    PreformContext,
};
use conform7_syntax::sentence::{break_sentences, SentenceClassification};
use conform7_syntax::verb_phrases::VerbPhrases;
use conform7_syntax::verbs::{
    VerbMeaning, Verbs, SVO_FS_BIT, VO_FS_BIT, SVOO_FS_BIT, VOO_FS_BIT,
};
use conform7_syntax::word_assemblage::WordAssemblage;
use conform7_syntax::Wording;
use std::io::{self, BufRead, Write};

// ---------------------------------------------------------------------------
// Verb registry setup
// ---------------------------------------------------------------------------

/// Build a verb registry with common English verbs for debugging.
fn build_verb_registry() -> Verbs {
    let mut v = Verbs::new();
    let cat = v.stock.new_category("verb");

    fn add_verb(
        v: &mut Verbs,
        cat: usize,
        infinitive: &str,
        copular: bool,
        forms: &[&str],
        form_structs: u8,
    ) {
        let verb = v.new_verb(None, copular);
        let infinitive_owned: Box<dyn std::any::Any> = Box::new(infinitive.to_string());
        v.add_form(
            verb,
            None,
            None,
            VerbMeaning::regular(infinitive_owned),
            form_structs,
        );
        let item = v.stock.add_item(cat, Box::new(verb));
        let usage = v.stock.new_usage(item, "English");
        let mut tier = None;
        for &form in forms {
            let wa = WordAssemblage::lit_1(form);
            if let Some(usage_ref) = v.new_usage(wa, false, usage, None) {
                if tier.is_none() {
                    tier = Some(v.new_tier(100));
                }
                if let Some(t) = tier {
                    v.add_usage_to_tier(usage_ref, t);
                }
            }
        }
    }

    add_verb(
        &mut v, cat, "to be", true,
        &["is", "are", "was", "were", "am", "be", "been", "being"],
        SVO_FS_BIT,
    );
    add_verb(
        &mut v, cat, "to have", false,
        &["has", "have", "had", "having"],
        SVO_FS_BIT | VO_FS_BIT,
    );
    add_verb(
        &mut v, cat, "to carry", false,
        &["carry", "carries", "carried", "carrying"],
        SVO_FS_BIT | VO_FS_BIT,
    );
    add_verb(
        &mut v, cat, "to take", false,
        &["take", "takes", "took", "taken", "taking"],
        SVO_FS_BIT | VO_FS_BIT,
    );
    add_verb(
        &mut v, cat, "to put", false,
        &["put", "puts", "putting"],
        SVO_FS_BIT | VO_FS_BIT | SVOO_FS_BIT | VOO_FS_BIT,
    );
    add_verb(
        &mut v, cat, "to look", false,
        &["look", "looks", "looked", "looking"],
        VO_FS_BIT,
    );
    add_verb(
        &mut v, cat, "to say", false,
        &["say", "says", "said", "saying"],
        SVO_FS_BIT | VO_FS_BIT,
    );
    add_verb(
        &mut v, cat, "to wear", false,
        &["wear", "wears", "wore", "worn", "wearing"],
        SVO_FS_BIT | VO_FS_BIT,
    );
    add_verb(
        &mut v, cat, "to try", false,
        &["try", "tries", "tried", "trying"],
        SVO_FS_BIT | VO_FS_BIT,
    );

    v
}

// ---------------------------------------------------------------------------
// Grammar
// ---------------------------------------------------------------------------

/// The synthetic Preform grammar used for debugging.
const DEBUG_GRAMMAR: &str = concat!(
    "<sentence> internal\n",
    "<nonimperative-verb> internal\n",
    "<negated-noncopular-verb-present> internal\n",
    "<np-unparsed> ::= ...\n",
);

// ---------------------------------------------------------------------------
// Output helpers
// ---------------------------------------------------------------------------

fn print_separator() {
    println!("{}", "-".repeat(72));
}

/// Get the visible (non-whitespace) words from a token range.
fn visible_words(tokens: &[conform7_syntax::Token], range: std::ops::Range<usize>) -> String {
    tokens[range]
        .iter()
        .filter(|t| {
            t.kind != conform7_syntax::SyntaxKind::WHITESPACE
                && t.kind != conform7_syntax::SyntaxKind::NEWLINE
        })
        .map(|t| t.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn print_tokens(source: &str) {
    let tokens = Lexer::tokenize(source).unwrap_or_else(|e| e);
    println!("  Tokens ({}):", tokens.len());
    for (i, token) in tokens.iter().enumerate() {
        let text_repr = if token.text.len() > 40 {
            format!("{:40}…", &token.text[..37])
        } else {
            format!("{:40}", token.text)
        };
        println!(
            "    {:>3}: {:12} {:?}",
            i,
            format!("{:?}", token.kind),
            text_repr.trim()
        );
    }
}

fn print_sentences(source: &str) {
    let tokens = Lexer::tokenize(source).unwrap_or_else(|e| e);
    let sentences = break_sentences(&tokens);
    println!("  Sentences ({}):", sentences.len());
    for (i, sent) in sentences.iter().enumerate() {
        let class_str = match &sent.classification {
            SentenceClassification::Heading { level } => format!("Heading({:?})", level),
            SentenceClassification::Structural(st) => format!("Structural({:?})", st),
            SentenceClassification::Regular => "Regular".to_string(),
            SentenceClassification::RulePreamble => "RulePreamble".to_string(),
            SentenceClassification::RulePhrase => "RulePhrase".to_string(),
        };
        let words = visible_words(&tokens, sent.token_range.clone());
        println!("    {:>3}: {} — {:?}", i, class_str, words);
    }
}

/// Format form structure bits as a readable string.
fn format_form_structures(bits: u8) -> String {
    let mut parts = Vec::new();
    if bits & SVO_FS_BIT != 0 {
        parts.push("SVO");
    }
    if bits & VO_FS_BIT != 0 {
        parts.push("VO");
    }
    if bits & SVOO_FS_BIT != 0 {
        parts.push("SVOO");
    }
    if bits & VOO_FS_BIT != 0 {
        parts.push("VOO");
    }
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join("|")
    }
}

/// Format a certainty level as a readable string.
fn format_certainty(level: i32) -> &'static str {
    match level {
        -2 => "IMPOSSIBLE",
        -1 => "UNLIKELY",
        0 => "UNKNOWN",
        1 => "LIKELY",
        2 => "CERTAIN",
        3 => "INITIALLY",
        _ => "?",
    }
}

/// Show annotations on a parse node.
fn show_annotations(node: &conform7_syntax::ParseNode, verbs: &Verbs) {
    let anns = node.annotations();
    if anns.is_empty() {
        return;
    }
    println!("      Annotations:");
    for ann in anns {
        match ann {
            Annotation::HeadingLevel(lvl) => {
                println!("        heading_level: {:?}", lvl);
            }
            Annotation::ArticleUsage(au) => {
                println!("        article: {:?} (word: {:?})", au.article.name, au.word);
            }
            Annotation::VerbalCertainty(level) => {
                println!("        verbal_certainty: {} ({})", level, format_certainty(*level));
            }
            Annotation::SentenceIsExistential(val) => {
                println!("        existential: {}", val);
            }
            Annotation::LinguisticErrorHere(code) => {
                println!("        linguistic_error: code={}", code);
            }
            Annotation::VerbUsage(vu_ref) => {
                if let Some(vu) = verbs.usages.get(*vu_ref) {
                    let vu_text = vu.vu_text.words.join(" ");
                    println!("        verb_usage[{}]: {:?}", vu_ref, vu_text);
                    if let Some(gu) = verbs.stock.usages.get(vu.usage) {
                        println!("          grammatical_usage: item={}, lang={:?}",
                            gu.item, gu.language);
                    }
                } else {
                    println!("        verb_usage[{}]: <invalid>", vu_ref);
                }
            }
            Annotation::PrepositionRef(prep) => {
                if let Some(p) = verbs.prepositions.get(*prep) {
                    let text = p.prep_text.words.join(" ");
                    println!("        preposition[{}]: {:?}", prep, text);
                } else {
                    println!("        preposition[{}]: <invalid>", prep);
                }
            }
            Annotation::SecondPrepositionRef(prep) => {
                if let Some(p) = verbs.prepositions.get(*prep) {
                    let text = p.prep_text.words.join(" ");
                    println!("        second_preposition[{}]: {:?}", prep, text);
                } else {
                    println!("        second_preposition[{}]: <invalid>", prep);
                }
            }
        }
    }
}

/// Show the viability map for a sentence.
fn show_viability_map(
    _sentence_text: &str,
    ctx: &PreformContext,
    registry: &InternalRegistry,
    tokens: &[conform7_syntax::Token],
) {
    let word_texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
    let word_count = word_texts.len();
    if word_count == 0 {
        return;
    }
    let wording = Wording::new(0, word_count as u32);
    let vm = VerbPhrases::calculate_viability_map(wording, ctx, registry);

    println!("    Viability map (0=not verb, 1=verb, 3=negated non-copular):");
    for (i, word) in word_texts.iter().enumerate() {
        let score = vm.get(i);
        if score > 0 && !word.trim().is_empty() {
            println!("      word {} ({:?}): score={}", i, word, score);
        }
    }
}

/// Show verb registry summary.
fn show_verb_registry(verbs: &Verbs) {
    println!("  Verb registry:");
    println!("    Verbs: {}", verbs.verbs.len());
    println!("    Forms: {}", verbs.forms.len());
    println!("    Usages: {}", verbs.usages.len());
    println!("    Tiers: {}", verbs.tiers.len());
    println!("    Prepositions: {}", verbs.prepositions.len());
    println!("    Conjugations: {}", verbs.conjugations.len());
    if let Some(cop) = verbs.copular_verb {
        println!("    Copular verb: index {}", cop);
    }

    // Show each verb and its forms.
    for (vi, verb) in verbs.verbs.iter().enumerate() {
        let is_copular = verbs.copular_verb == Some(vi);
        println!("    Verb[{}]:{}", vi, if is_copular { " (copular)" } else { "" });
        if let Some(conj) = verb.conjugation {
            if let Some(c) = verbs.conjugations.get(conj) {
                println!("      conjugation[{}]: infinitive={:?}", conj, c.infinitive.words.join(" "));
            }
        }
        // Show forms.
        let mut form_ref = verb.first_form;
        while let Some(fr) = form_ref {
            if let Some(form) = verbs.forms.get(fr) {
                let prep_text = form
                    .preposition
                    .and_then(|p| verbs.prepositions.get(p))
                    .map(|p| p.prep_text.words.join(" "))
                    .unwrap_or_default();
                println!(
                    "      form[{}]: structures={}, prep={:?}",
                    fr,
                    format_form_structures(form.form_structures),
                    if prep_text.is_empty() { "none" } else { &prep_text },
                );
            }
            form_ref = form_ref.and_then(|fr| verbs.forms.get(fr)).and_then(|f| f.next_form);
        }
    }

    // Show usages grouped by verb.
    for (ui, usage) in verbs.usages.iter().enumerate() {
        let text = usage.vu_text.words.join(" ");
        if !text.is_empty() {
            // Find which verb this usage belongs to by following the stock chain.
            if let Some(gu) = verbs.stock.usages.get(usage.usage) {
                println!("    Usage[{}]: {:?} (item={})", ui, text, gu.item);
            }
        }
    }
}

fn print_parse(sentence_text: &str, verbs: &Verbs) {
    let grammar = match parse_preform_grammar(DEBUG_GRAMMAR) {
        Ok(g) => g,
        Err(e) => {
            println!("    [Grammar error: {}]", e);
            return;
        }
    };

    // Tokenize the sentence to get word boundaries.
    let tokens = Lexer::tokenize(sentence_text).unwrap_or_else(|e| e);
    let word_texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
    let word_count = word_texts.len();
    if word_count == 0 {
        println!("    [Empty sentence]");
        return;
    }

    let ctx = PreformContext {
        grammar: &grammar,
        word_text: &word_texts,
        is_paragraph_start: false,
        verbs_registry: Some(verbs),
    };
    let registry = InternalRegistry::linguistics();

    // Show viability map.
    show_viability_map(sentence_text, &ctx, &registry, &tokens);

    // Try <sentence>.
    let wording = Wording::new(0, word_count as u32);
    let result = match_nonterminal_impl(&ctx, &registry, "sentence", wording);

    match result {
        Some(m) => {
            if let Some(internal) = &m.internal {
                match &internal.payload {
                    InternalPayload::ParseNode(node) => {
                        println!("    Parse tree:");
                        for line in format!("{}", node).lines() {
                            println!("      {}", line);
                        }

                        // Show annotations on the VERB_NT.
                        show_annotations(node, verbs);

                        // Show children in more detail.
                        if node.child_count() > 0 {
                            println!("    Children:");
                            for child in node.children() {
                                let w = child.wording();
                                let words =
                                    visible_words(&tokens, w.start as usize..w.end as usize);
                                println!(
                                    "      {} [{}..{}]: {:?}",
                                    child.node_type(),
                                    w.start,
                                    w.end,
                                    words
                                );
                                show_annotations(child, verbs);
                            }
                        }

                        // Show alternatives.
                        let alts: Vec<_> = node.alternatives().collect();
                        if !alts.is_empty() {
                            println!("    Alternatives ({}):", alts.len());
                            for alt in alts {
                                let w = alt.wording();
                                let words =
                                    visible_words(&tokens, w.start as usize..w.end as usize);
                                println!(
                                    "      {} [{}..{}]: {:?}",
                                    alt.node_type(),
                                    w.start,
                                    w.end,
                                    words
                                );
                                show_annotations(alt, verbs);
                            }
                        }
                    }
                    _ => {
                        println!("    Internal match: {:?}", internal.payload);
                    }
                }
            } else {
                println!(
                    "    Matched production {} (match number {})",
                    m.production_index, m.match_number
                );
                println!("    Word range: {}..{}", m.word_range.start, m.word_range.end);
            }
        }
        None => {
            println!("    [No match — sentence not recognized]");
        }
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn process_source(source: &str, verbs: &Verbs) {
    print_separator();
    println!("Input:");
    for line in source.lines() {
        println!("  {}", line);
    }
    println!();

    // 0. Verb registry summary
    println!("── Verb Registry ──");
    show_verb_registry(verbs);
    println!();

    // 1. Lexer
    println!("── Lexer ──");
    print_tokens(source);
    println!();

    // 2. Sentence breaker
    println!("── Sentences ──");
    print_sentences(source);
    println!();

    // 3. Parse each sentence
    println!("── Parse ──");
    let tokens = Lexer::tokenize(source).unwrap_or_else(|e| e);
    let sentences = break_sentences(&tokens);
    for (i, sent) in sentences.iter().enumerate() {
        let sentence_text = visible_words(&tokens, sent.token_range.clone());
        println!("  Sentence {}: {:?}", i, sentence_text);
        print_parse(&sentence_text, verbs);
        println!();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let verbs = build_verb_registry();

    if args.len() > 1 {
        // File mode
        let path = &args[1];
        match std::fs::read_to_string(path) {
            Ok(source) => process_source(&source, &verbs),
            Err(e) => {
                eprintln!("Error reading {}: {}", path, e);
                std::process::exit(1);
            }
        }
    } else {
        // Check if stdin has data (piped input) or is a terminal (REPL)
        let stdin = io::stdin();
        let has_piped_input = !stdin.lock().fill_buf().unwrap().is_empty();

        if has_piped_input {
            // Piped input
            let mut source = String::new();
            for line in stdin.lock().lines() {
                match line {
                    Ok(l) => {
                        source.push_str(&l);
                        source.push('\n');
                    }
                    Err(e) => {
                        eprintln!("Error reading stdin: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            process_source(&source, &verbs);
        } else {
            // Interactive REPL
            println!("Conform7 Debug REPL");
            println!("Enter Inform 7 source text (empty line to quit):");
            loop {
                print!("> ");
                io::stdout().flush().unwrap();
                let mut line = String::new();
                match io::stdin().read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        let line = line.trim();
                        if line.is_empty() {
                            break;
                        }
                        process_source(line, &verbs);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        break;
                    }
                }
            }
        }
    }
}
