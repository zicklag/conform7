use crate::knowledge::inference_subjects::InferenceSubject;

/// Certainty levels for inferences.
///
/// Corresponds to the `*_CE` constants in the C reference
/// (`inform7/knowledge-module/Chapter 5/Inferences.w`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Certainty {
    /// Known to be false.
    Impossible = -2,
    /// Unlikely to be true.
    Unlikely = -1,
    /// No information.
    Unknown = 0,
    /// Likely to be true.
    Likely = 1,
    /// Initially true (for start-of-play state).
    Initially = 2,
    /// Certainly true.
    Certain = 3,
}

/// Comparison result values for inference comparison.
///
/// Corresponds to the `CI_*` constants in the C reference
/// (`inform7/knowledge-module/Chapter 5/Inferences.w`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InferenceComparison {
    /// One exists, the other doesn't.
    DifferInExistence = 1,
    /// Different families.
    DifferInFamily = 2,
    /// Different topics.
    DifferInTopic = 3,
    /// Different Boolean content.
    DifferInBooleanContent = 4,
    /// Different content.
    DifferInContent = 5,
    /// Different but duplicate inferences.
    DifferInCopyOnly = 6,
    /// Pointers to the same inference.
    Identical = 0,
}

/// A single fact about the world model.
///
/// Corresponds to `inference` in the C reference
/// (`inform7/knowledge-module/Chapter 5/Inferences.w`, lines 10-18).
#[derive(Clone, Debug)]
pub struct Inference {
    /// The family this inference belongs to.
    pub family: usize, // simplified: index into a family registry
    /// Family-specific data (simplified: a string tag for now).
    pub data: Option<&'static str>,
    /// Index into a family-specific data registry (e.g., PropertyInferenceData).
    pub data_index: Option<usize>,
    /// The certainty of this inference.
    pub certainty: Certainty,
    /// Where this inference was drawn from (simplified: a string tag).
    pub inferred_from: Option<&'static str>,
    /// The building stage during which this was drawn.
    pub drawn_during_stage: i32,
    /// Whether this was drawn from the project's metadata file.
    pub drawn_from_metadata: bool,
}

/// A family of related inferences.
///
/// Corresponds to `inference_family` in the C reference
/// (`inform7/knowledge-module/Chapter 5/Inferences.w`, lines 345-349).
#[derive(Clone, Debug)]
pub struct InferenceFamily {
    /// Name of this family (for debugging).
    pub name: &'static str,
    /// Method implementations for this family.
    pub methods: InferenceFamilyMethods,
}

/// Methods that can be implemented for an inference family.
///
/// Corresponds to the method dispatch table in the C reference
/// (`inform7/knowledge-module/Chapter 5/Inferences.w`, lines 340-401).
#[derive(Clone, Debug)]
pub struct InferenceFamilyMethods {
    /// Log family-specific details.
    pub log_details: fn(&Inference) -> String,
    /// Compare two inferences from this family.
    pub compare: fn(&Inference, &Inference) -> i32,
    /// Explain a contradiction (returns true if handled).
    pub explain_contradiction: fn(&Inference, &Inference, i32, usize) -> bool,
}

/// Result of joining an inference to a subject.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinResult {
    /// Inference was added to the list.
    Joined,
    /// Inference replaced an existing one.
    Replaced,
    /// Inference was discarded as redundant.
    DiscardedRedundant,
    /// Inference was discarded as a harmless contradiction.
    DiscardedContradiction,
    /// Inference was discarded because we already know better.
    DiscardedWeaker,
}

impl Inference {
    /// Create a new inference.
    ///
    /// If the certainty is `Unknown`, it defaults to `Certain`.
    ///
    /// Corresponds to `Inferences::create_inference` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Inferences.w`, lines 25-38).
    pub fn new(family: usize, data: Option<&'static str>, certainty: Certainty) -> Self {
        let certainty = if certainty == Certainty::Unknown {
            Certainty::Certain
        } else {
            certainty
        };
        Inference {
            family,
            data,
            data_index: None,
            certainty,
            inferred_from: None,
            drawn_during_stage: 0,
            drawn_from_metadata: false,
        }
    }

    /// Get the certainty of this inference.
    ///
    /// Corresponds to `Inferences::get_certainty` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Inferences.w`, lines 45-67).
    pub fn get_certainty(&self) -> Certainty {
        self.certainty
    }

    /// Get where this inference was drawn from.
    ///
    /// Corresponds to `Inferences::where_inferred` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Inferences.w`, lines 45-67).
    pub fn where_inferred(&self) -> Option<&'static str> {
        self.inferred_from
    }

    /// Get the family index of this inference.
    ///
    /// Corresponds to `Inferences::get_inference_type` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Inferences.w`, lines 45-67).
    pub fn get_family(&self) -> usize {
        self.family
    }

    /// Mark this inference as impossible.
    ///
    /// Corresponds to `Inferences::render_impossible` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Inferences.w`).
    pub fn render_impossible(&mut self) {
        self.certainty = Certainty::Impossible;
    }

    /// Compare this inference with another.
    ///
    /// Returns the appropriate `InferenceComparison` value.
    ///
    /// Corresponds to `Inferences::cmp` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Inferences.w`, lines 112-127).
    pub fn cmp(&self, other: &Inference, families: &[InferenceFamily]) -> InferenceComparison {
        if self.family != other.family {
            return InferenceComparison::DifferInFamily;
        }
        if self.data != other.data {
            return InferenceComparison::DifferInTopic;
        }
        // If families have a custom comparison, use it
        if self.family < families.len() {
            let cmp_result = (families[self.family].methods.compare)(self, other);
            if cmp_result != 0 {
                return match cmp_result {
                    1 => InferenceComparison::DifferInExistence,
                    2 => InferenceComparison::DifferInFamily,
                    3 => InferenceComparison::DifferInTopic,
                    4 => InferenceComparison::DifferInBooleanContent,
                    5 => InferenceComparison::DifferInContent,
                    6 => InferenceComparison::DifferInCopyOnly,
                    _ => InferenceComparison::DifferInContent,
                };
            }
        }
        InferenceComparison::Identical
    }

    /// Join this inference to a subject's inference list.
    ///
    /// Attempts to add this inference to the subject's list, handling
    /// contradictions and duplicates.
    ///
    /// Corresponds to `Inferences::join_inference` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Inferences.w`, lines 180-199).
    pub fn join(
        &self,
        subject: &mut InferenceSubject,
        families: &[InferenceFamily],
        inferences: &mut Vec<Inference>,
    ) -> JoinResult {
        // Check for existing inferences with the same data (using cmp)
        for i in 0..subject.inf_list.len() {
            let existing_idx = subject.inf_list[i];
            if existing_idx < inferences.len() {
                let existing = &inferences[existing_idx];
                let cmp = self.cmp(existing, families);
                if cmp == InferenceComparison::Identical {
                    // Same inference data — handle certainty
                    if self.certainty > existing.certainty {
                        // Replace with higher certainty
                        inferences[existing_idx].certainty = self.certainty;
                        return JoinResult::Replaced;
                    } else if self.certainty < existing.certainty {
                        // Existing is stronger — discard
                        return JoinResult::DiscardedWeaker;
                    } else if self.certainty == Certainty::Impossible
                        && existing.certainty == Certainty::Impossible
                    {
                        // Both impossible — redundant
                        return JoinResult::DiscardedRedundant;
                    } else {
                        // Same certainty, same data — redundant
                        return JoinResult::DiscardedRedundant;
                    }
                }
            }
        }

        // No conflict — add to the registry and the list
        let idx = inferences.len();
        inferences.push(self.clone());
        subject.inf_list.push(idx);
        JoinResult::Joined
    }
}

impl InferenceFamily {
    /// Create a new inference family.
    ///
    /// Corresponds to the family creation in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Inferences.w`, lines 345-349).
    pub fn new(name: &'static str, methods: InferenceFamilyMethods) -> Self {
        InferenceFamily { name, methods }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::inference_subjects::InferenceSubject;

    fn make_family() -> InferenceFamily {
        InferenceFamily::new(
            "test",
            InferenceFamilyMethods {
                log_details: |inf| format!("log({:?})", inf.data),
                compare: |a, b| {
                    if a.data == b.data {
                        0
                    } else {
                        5 // DifferInContent
                    }
                },
                explain_contradiction: |_, _, _, _| false,
            },
        )
    }

    #[test]
    fn test_new_creates_inference_with_correct_family() {
        let inf = Inference::new(0, Some("test_data"), Certainty::Certain);
        assert_eq!(inf.family, 0);
        assert_eq!(inf.data, Some("test_data"));
        assert_eq!(inf.certainty, Certainty::Certain);
    }

    #[test]
    fn test_new_defaults_unknown_to_certain() {
        let inf = Inference::new(0, None, Certainty::Unknown);
        assert_eq!(inf.certainty, Certainty::Certain);
    }

    #[test]
    fn test_new_preserves_non_unknown_certainty() {
        let inf = Inference::new(0, None, Certainty::Impossible);
        assert_eq!(inf.certainty, Certainty::Impossible);

        let inf = Inference::new(0, None, Certainty::Unlikely);
        assert_eq!(inf.certainty, Certainty::Unlikely);

        let inf = Inference::new(0, None, Certainty::Likely);
        assert_eq!(inf.certainty, Certainty::Likely);

        let inf = Inference::new(0, None, Certainty::Initially);
        assert_eq!(inf.certainty, Certainty::Initially);

        let inf = Inference::new(0, None, Certainty::Certain);
        assert_eq!(inf.certainty, Certainty::Certain);
    }

    #[test]
    fn test_get_certainty_returns_correct_value() {
        let inf = Inference::new(0, None, Certainty::Likely);
        assert_eq!(inf.get_certainty(), Certainty::Likely);
    }

    #[test]
    fn test_where_inferred_returns_none_by_default() {
        let inf = Inference::new(0, None, Certainty::Certain);
        assert_eq!(inf.where_inferred(), None);
    }

    #[test]
    fn test_where_inferred_returns_set_value() {
        let mut inf = Inference::new(0, None, Certainty::Certain);
        inf.inferred_from = Some("test_sentence");
        assert_eq!(inf.where_inferred(), Some("test_sentence"));
    }

    #[test]
    fn test_get_family_returns_correct_index() {
        let inf = Inference::new(42, None, Certainty::Certain);
        assert_eq!(inf.get_family(), 42);
    }

    #[test]
    fn test_render_impossible_sets_certainty() {
        let mut inf = Inference::new(0, None, Certainty::Certain);
        assert_eq!(inf.certainty, Certainty::Certain);
        inf.render_impossible();
        assert_eq!(inf.certainty, Certainty::Impossible);
    }

    #[test]
    fn test_cmp_identical_for_same_inference() {
        let families = vec![make_family()];
        let inf = Inference::new(0, Some("data"), Certainty::Certain);
        assert_eq!(inf.cmp(&inf, &families), InferenceComparison::Identical);
    }

    #[test]
    fn test_cmp_differ_in_family() {
        let families = vec![make_family(), make_family()];
        let a = Inference::new(0, Some("data"), Certainty::Certain);
        let b = Inference::new(1, Some("data"), Certainty::Certain);
        assert_eq!(a.cmp(&b, &families), InferenceComparison::DifferInFamily);
    }
    #[test]
    fn test_join_adds_to_empty_list() {
        let families = vec![make_family()];
        let inf = Inference::new(0, Some("data"), Certainty::Certain);
        let mut subject = InferenceSubject::new(0, None, None, None);
        let mut inferences = Vec::new();

        assert_eq!(
            inf.join(&mut subject, &families, &mut inferences),
            JoinResult::Joined
        );
        assert_eq!(subject.inf_list.len(), 1);
        assert_eq!(inferences.len(), 1);
        assert_eq!(inferences[0].data, Some("data"));
    }

    #[test]
    fn test_join_discards_redundant() {
        let families = vec![make_family()];
        let inf = Inference::new(0, Some("data"), Certainty::Certain);
        let mut subject = InferenceSubject::new(0, None, None, None);
        let mut inferences = vec![Inference::new(0, Some("data"), Certainty::Certain)];
        subject.inf_list.push(0); // existing inference with same data

        assert_eq!(
            inf.join(&mut subject, &families, &mut inferences),
            JoinResult::DiscardedRedundant
        );
        assert_eq!(subject.inf_list.len(), 1);
    }

    #[test]
    fn test_inference_family_new() {
        let family = make_family();
        assert_eq!(family.name, "test");
    }

    #[test]
    fn test_inference_family_methods_log_details() {
        let family = make_family();
        let inf = Inference::new(0, Some("hello"), Certainty::Certain);
        let log = (family.methods.log_details)(&inf);
        assert_eq!(log, "log(Some(\"hello\"))");
    }

    #[test]
    fn test_inference_family_methods_compare() {
        let family = make_family();
        let a = Inference::new(0, Some("same"), Certainty::Certain);
        let b = Inference::new(0, Some("same"), Certainty::Certain);
        assert_eq!((family.methods.compare)(&a, &b), 0);

        let c = Inference::new(0, Some("different"), Certainty::Certain);
        assert_eq!((family.methods.compare)(&a, &c), 5);
    }

    #[test]
    fn test_certainty_ordering() {
        assert!(Certainty::Impossible < Certainty::Unlikely);
        assert!(Certainty::Unlikely < Certainty::Unknown);
        assert!(Certainty::Unknown < Certainty::Likely);
        assert!(Certainty::Likely < Certainty::Initially);
        assert!(Certainty::Initially < Certainty::Certain);
    }

    #[test]
    fn test_certainty_values() {
        assert_eq!(Certainty::Impossible as i8, -2);
        assert_eq!(Certainty::Unlikely as i8, -1);
        assert_eq!(Certainty::Unknown as i8, 0);
        assert_eq!(Certainty::Likely as i8, 1);
        assert_eq!(Certainty::Initially as i8, 2);
        assert_eq!(Certainty::Certain as i8, 3);
    }

    #[test]
    fn test_drawn_during_stage_default() {
        let inf = Inference::new(0, None, Certainty::Certain);
        assert_eq!(inf.drawn_during_stage, 0);
    }

    #[test]
    fn test_drawn_from_metadata_default() {
        let inf = Inference::new(0, None, Certainty::Certain);
        assert!(!inf.drawn_from_metadata);
    }

    #[test]
    fn test_inference_comparison_values() {
        assert_eq!(InferenceComparison::DifferInExistence as i8, 1);
        assert_eq!(InferenceComparison::DifferInFamily as i8, 2);
        assert_eq!(InferenceComparison::DifferInTopic as i8, 3);
        assert_eq!(InferenceComparison::DifferInBooleanContent as i8, 4);
        assert_eq!(InferenceComparison::DifferInContent as i8, 5);
        assert_eq!(InferenceComparison::DifferInCopyOnly as i8, 6);
        assert_eq!(InferenceComparison::Identical as i8, 0);
    }

    #[test]
    fn test_join_replaces_with_higher_certainty() {
        let families = vec![make_family()];
        let mut subject = InferenceSubject::new(0, None, None, None);
        let mut inferences = vec![Inference::new(0, Some("data"), Certainty::Likely)];
        subject.inf_list.push(0);

        let inf = Inference::new(0, Some("data"), Certainty::Certain);
        assert_eq!(
            inf.join(&mut subject, &families, &mut inferences),
            JoinResult::Replaced
        );
        assert_eq!(inferences[0].certainty, Certainty::Certain);
        assert_eq!(subject.inf_list.len(), 1);
    }

    #[test]
    fn test_join_discards_weaker() {
        let families = vec![make_family()];
        let mut subject = InferenceSubject::new(0, None, None, None);
        let mut inferences = vec![Inference::new(0, Some("data"), Certainty::Certain)];
        subject.inf_list.push(0);

        let inf = Inference::new(0, Some("data"), Certainty::Likely);
        assert_eq!(
            inf.join(&mut subject, &families, &mut inferences),
            JoinResult::DiscardedWeaker
        );
        assert_eq!(inferences[0].certainty, Certainty::Certain);
        assert_eq!(subject.inf_list.len(), 1);
    }

    #[test]
    fn test_join_handles_different_data() {
        let families = vec![make_family()];
        let mut subject = InferenceSubject::new(0, None, None, None);
        let mut inferences = vec![Inference::new(0, Some("existing"), Certainty::Certain)];
        subject.inf_list.push(0);

        let inf = Inference::new(0, Some("new_data"), Certainty::Certain);
        assert_eq!(
            inf.join(&mut subject, &families, &mut inferences),
            JoinResult::Joined
        );
        assert_eq!(subject.inf_list.len(), 2);
        assert_eq!(inferences.len(), 2);
    }

    #[test]
    fn test_join_stores_inference_index_not_family() {
        let families = vec![make_family()];
        let mut subject = InferenceSubject::new(0, None, None, None);
        let mut inferences = Vec::new();

        let inf1 = Inference::new(0, Some("data1"), Certainty::Certain);
        let inf2 = Inference::new(0, Some("data2"), Certainty::Certain);

        inf1.join(&mut subject, &families, &mut inferences);
        inf2.join(&mut subject, &families, &mut inferences);

        // inf_list should contain inference indices (0 and 1), not family indices
        assert_eq!(subject.inf_list, vec![0, 1]);
        assert_eq!(inferences.len(), 2);
    }
}

