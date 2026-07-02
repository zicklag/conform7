/// Maximum number of plugins that can attach data to a subject.
///
/// Corresponds to `MAX_PLUGINS` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`).
pub const MAX_PLUGINS: usize = 8;

/// An inference subject — anything about which an inference can be drawn.
///
/// Corresponds to `inference_subject` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 111-128).
///
/// Subjects form a hierarchy (a DAG) where narrower subjects inherit from
/// broader ones. The top of the hierarchy is the `model_world` subject.
#[derive(Clone, Debug)]
pub struct InferenceSubject {
    /// The broader (more general) subject in the hierarchy, or None for the root.
    pub broader_than: Option<usize>, // simplified: index into a subject registry
    /// The family this subject belongs to.
    pub infs_family: usize, // simplified: index into a family registry
    /// Family-specific data (simplified: a string tag for now).
    pub represents: Option<&'static str>,
    /// List of inferences drawn about this subject (contingently true).
    pub inf_list: Vec<usize>, // simplified: indices into an inference registry
    /// List of implications applying to this subject (necessarily true).
    pub imp_list: Vec<usize>, // simplified: indices into an implication registry
    /// List of property permissions for this subject.
    pub permissions_list: Vec<usize>, // simplified: indices into a permission registry
    /// Alias variable (for "player" aliasing "yourself").
    pub alias_variable: Option<&'static str>,
    /// Log name for debugging.
    pub log_name: Option<&'static str>,
}

/// A family of related inference subjects.
///
/// Corresponds to `inference_subject_family` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 15-18).
#[derive(Clone, Debug)]
pub struct InferenceSubjectFamily {
    /// Name of this family (for debugging).
    pub name: &'static str,
    /// Method implementations for this family.
    pub methods: InferenceSubjectFamilyMethods,
}

/// Methods that can be implemented for an inference subject family.
///
/// Corresponds to the method dispatch table in the C reference
/// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 14-24).
#[derive(Clone, Debug)]
pub struct InferenceSubjectFamilyMethods {
    /// Get the name text of a subject.
    pub get_name_text: fn(&InferenceSubject) -> Option<&'static str>,
    /// Get the default certainty for a subject.
    pub get_default_certainty: fn(&InferenceSubject) -> i8,
    /// Called when a new property permission is granted.
    pub new_permission_granted: fn(&InferenceSubject, usize),
    /// Called to make a subject the domain of an adjectival constant.
    pub make_adj_const_domain: fn(&InferenceSubject, usize, usize),
    /// Called during model completion.
    pub complete_model: fn(&InferenceSubject),
    /// Called during model checking.
    pub check_model: fn(&InferenceSubject),
}

impl InferenceSubject {
    /// Create a new inference subject.
    ///
    /// Corresponds to `InferenceSubjects::new` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 99-165).
    pub fn new(
        family: usize,
        broader_than: Option<usize>,
        represents: Option<&'static str>,
        log_name: Option<&'static str>,
    ) -> Self {
        InferenceSubject {
            broader_than,
            infs_family: family,
            represents,
            inf_list: Vec::new(),
            imp_list: Vec::new(),
            permissions_list: Vec::new(),
            alias_variable: None,
            log_name,
        }
    }

    /// Create a fundamental inference subject (model_world, global_variables, etc.).
    ///
    /// Fundamental subjects use the fundamentals family (index 0).
    ///
    /// Corresponds to `InferenceSubjects::new_fundamental` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 99-165).
    pub fn new_fundamental(broader_than: Option<usize>, log_name: &'static str) -> Self {
        InferenceSubject {
            broader_than,
            infs_family: 0, // fundamentals family is always at index 0
            represents: None,
            inf_list: Vec::new(),
            imp_list: Vec::new(),
            permissions_list: Vec::new(),
            alias_variable: None,
            log_name: Some(log_name),
        }
    }

    /// Test if this subject is within (contained by) another subject.
    ///
    /// Follows the subject hierarchy upward from this subject to the root,
    /// returning true if `larger` is found along the path.
    ///
    /// Corresponds to `InferenceSubjects::is_within` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 210-242).
    pub fn is_within(&self, larger: &InferenceSubject, registry: &[InferenceSubject]) -> bool {
        // Find the index of `larger` in the registry by pointer identity
        let larger_idx = match registry.iter().position(|s| std::ptr::eq(s, larger)) {
            Some(idx) => idx,
            None => return false,
        };

        // Walk the broader_than chain from self upward
        let mut current = self.broader_than;
        while let Some(idx) = current {
            if idx == larger_idx {
                return true;
            }
            current = registry[idx].broader_than;
        }
        false
    }

    /// Test if this subject is strictly within another (not the same subject).
    ///
    /// Corresponds to `InferenceSubjects::is_strictly_within` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 210-242).
    pub fn is_strictly_within(&self, larger: &InferenceSubject, registry: &[InferenceSubject]) -> bool {
        // A subject is not strictly within itself
        if std::ptr::eq(self, larger) {
            return false;
        }
        self.is_within(larger, registry)
    }

    /// Return the immediate broader subject index, if any.
    ///
    /// Corresponds to `InferenceSubjects::narrowest_broader_subject` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 210-242).
    pub fn narrowest_broader_subject(&self) -> Option<usize> {
        self.broader_than
    }

    /// Demote this subject to a new broader subject.
    ///
    /// Corresponds to `InferenceSubjects::falls_within` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 210-242).
    pub fn falls_within(&mut self, broad: usize) {
        self.broader_than = Some(broad);
    }

    /// Get the list of inference indices for this subject.
    ///
    /// Corresponds to `InferenceSubjects::get_inferences` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 247-267).
    pub fn get_inferences(&self) -> &[usize] {
        &self.inf_list
    }

    /// Get the list of implication indices for this subject.
    ///
    /// Corresponds to `InferenceSubjects::get_implications` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 247-267).
    pub fn get_implications(&self) -> &[usize] {
        &self.imp_list
    }

    /// Get the list of permission indices for this subject.
    ///
    /// Corresponds to `InferenceSubjects::get_permissions` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 247-267).
    pub fn get_permissions(&self) -> &[usize] {
        &self.permissions_list
    }

    /// Get the name text of this subject via family method dispatch.
    ///
    /// Corresponds to `InferenceSubjects::get_name_text` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 365-463).
    pub fn get_name_text(&self, families: &[InferenceSubjectFamily]) -> Option<&'static str> {
        if self.infs_family < families.len() {
            (families[self.infs_family].methods.get_name_text)(self)
        } else {
            None
        }
    }

    /// Get the default certainty for this subject via family method dispatch.
    ///
    /// Corresponds to `InferenceSubjects::get_default_certainty` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 365-463).
    pub fn get_default_certainty(&self, families: &[InferenceSubjectFamily]) -> i8 {
        if self.infs_family < families.len() {
            (families[self.infs_family].methods.get_default_certainty)(self)
        } else {
            0 // Unknown
        }
    }

    /// Notify the family that a new property permission has been granted.
    ///
    /// Corresponds to `InferenceSubjects::new_permission_granted` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 365-463).
    pub fn new_permission_granted(&self, families: &[InferenceSubjectFamily], pp: usize) {
        if self.infs_family < families.len() {
            (families[self.infs_family].methods.new_permission_granted)(self, pp);
        }
    }
}

impl InferenceSubjectFamily {
    /// Create a new inference subject family.
    ///
    /// Corresponds to the family creation in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 15-18).
    pub fn new(name: &'static str, methods: InferenceSubjectFamilyMethods) -> Self {
        InferenceSubjectFamily { name, methods }
    }

    /// Create the fundamentals family with default method implementations.
    ///
    /// The fundamentals family is used for fundamental subjects like
    /// `model_world`, `global_variables`, etc.
    ///
    /// Corresponds to the fundamentals family in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 35-55).
    pub fn fundamentals() -> Self {
        InferenceSubjectFamily {
            name: "fundamentals",
            methods: InferenceSubjectFamilyMethods {
                get_name_text: |_| None,
                get_default_certainty: |_| 3, // CERTAIN_CE
                new_permission_granted: |_, _| {},
                make_adj_const_domain: |_, _, _| {},
                complete_model: |_| {},
                check_model: |_| {},
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_families() -> Vec<InferenceSubjectFamily> {
        vec![InferenceSubjectFamily::fundamentals()]
    }

    #[test]
    fn test_new_creates_subject_with_correct_fields() {
        let subject = InferenceSubject::new(0, None, Some("test_rep"), Some("test_subject"));

        assert_eq!(subject.infs_family, 0);
        assert_eq!(subject.broader_than, None);
        assert_eq!(subject.represents, Some("test_rep"));
        assert_eq!(subject.log_name, Some("test_subject"));
        assert!(subject.inf_list.is_empty());
        assert!(subject.imp_list.is_empty());
        assert!(subject.permissions_list.is_empty());
        assert_eq!(subject.alias_variable, None);
    }

    #[test]
    fn test_new_fundamental_creates_fundamental_subject() {
        let subject = InferenceSubject::new_fundamental(None, "model_world");

        assert_eq!(subject.infs_family, 0); // fundamentals family
        assert_eq!(subject.broader_than, None);
        assert_eq!(subject.represents, None);
        assert_eq!(subject.log_name, Some("model_world"));
    }

    #[test]
    fn test_new_fundamental_with_broader() {
        let _root = InferenceSubject::new_fundamental(None, "model_world");
        let child = InferenceSubject::new_fundamental(Some(0), "global_variables");

        assert_eq!(child.broader_than, Some(0));
        assert_eq!(child.log_name, Some("global_variables"));
    }

    #[test]
    fn test_narrowest_broader_subject_returns_broader() {
        let subject = InferenceSubject::new(0, Some(5), None, None);
        assert_eq!(subject.narrowest_broader_subject(), Some(5));
    }

    #[test]
    fn test_narrowest_broader_subject_none_for_root() {
        let subject = InferenceSubject::new(0, None, None, None);
        assert_eq!(subject.narrowest_broader_subject(), None);
    }

    #[test]
    fn test_falls_within_demotes_subject() {
        let mut subject = InferenceSubject::new(0, None, None, None);
        assert_eq!(subject.broader_than, None);

        subject.falls_within(3);
        assert_eq!(subject.broader_than, Some(3));
    }

    #[test]
    fn test_get_inferences_returns_empty_list() {
        let subject = InferenceSubject::new(0, None, None, None);
        assert!(subject.get_inferences().is_empty());
    }

    #[test]
    fn test_get_implications_returns_empty_list() {
        let subject = InferenceSubject::new(0, None, None, None);
        assert!(subject.get_implications().is_empty());
    }

    #[test]
    fn test_get_permissions_returns_empty_list() {
        let subject = InferenceSubject::new(0, None, None, None);
        assert!(subject.get_permissions().is_empty());
    }

    #[test]
    fn test_get_name_text_dispatches_to_family() {
        let families = make_families();
        let subject = InferenceSubject::new(0, None, None, None);
        assert_eq!(subject.get_name_text(&families), None);
    }

    #[test]
    fn test_get_default_certainty_dispatches_to_family() {
        let families = make_families();
        let subject = InferenceSubject::new(0, None, None, None);
        assert_eq!(subject.get_default_certainty(&families), 3); // CERTAIN_CE
    }

    #[test]
    fn test_new_permission_granted_dispatches_to_family() {
        let families = make_families();
        let subject = InferenceSubject::new(0, None, None, None);
        // Should not panic
        subject.new_permission_granted(&families, 42);
    }

    #[test]
    fn test_method_dispatch_with_invalid_family_index() {
        let families = make_families();
        let subject = InferenceSubject::new(999, None, None, None);
        assert_eq!(subject.get_name_text(&families), None);
        assert_eq!(subject.get_default_certainty(&families), 0); // Unknown
        // Should not panic
        subject.new_permission_granted(&families, 0);
    }

    #[test]
    fn test_fundamentals_family_defaults() {
        let family = InferenceSubjectFamily::fundamentals();
        assert_eq!(family.name, "fundamentals");

        let subject = InferenceSubject::new(0, None, None, None);
        assert_eq!((family.methods.get_name_text)(&subject), None);
        assert_eq!((family.methods.get_default_certainty)(&subject), 3);
    }

    #[test]
    fn test_custom_family_methods() {
        let custom_methods = InferenceSubjectFamilyMethods {
            get_name_text: |s| s.log_name,
            get_default_certainty: |_| 1, // LIKELY_CE
            new_permission_granted: |_, _| {},
            make_adj_const_domain: |_, _, _| {},
            complete_model: |_| {},
            check_model: |_| {},
        };
        let family = InferenceSubjectFamily::new("custom", custom_methods);
        let families = vec![family];

        let subject = InferenceSubject::new(0, None, None, Some("my_subject"));
        assert_eq!(subject.get_name_text(&families), Some("my_subject"));
        assert_eq!(subject.get_default_certainty(&families), 1);
    }

    #[test]
    fn test_is_within_self_is_not_strictly_within() {
        let registry = vec![
            InferenceSubject::new(0, None, None, Some("root")),
        ];
        assert!(!registry[0].is_strictly_within(&registry[0], &registry));
    }

    #[test]
    fn test_get_inferences_with_items() {
        let mut subject = InferenceSubject::new(0, None, None, None);
        subject.inf_list.push(1);
        subject.inf_list.push(2);
        subject.inf_list.push(3);
        assert_eq!(subject.get_inferences(), &[1, 2, 3]);
    }

    #[test]
    fn test_get_implications_with_items() {
        let mut subject = InferenceSubject::new(0, None, None, None);
        subject.imp_list.push(10);
        subject.imp_list.push(20);
        assert_eq!(subject.get_implications(), &[10, 20]);
    }

    #[test]
    fn test_get_permissions_with_items() {
        let mut subject = InferenceSubject::new(0, None, None, None);
        subject.permissions_list.push(100);
        assert_eq!(subject.get_permissions(), &[100]);
    }

    #[test]
    fn test_alias_variable() {
        let mut subject = InferenceSubject::new(0, None, None, Some("player"));
        subject.alias_variable = Some("yourself");
        assert_eq!(subject.alias_variable, Some("yourself"));
    }

    #[test]
    fn test_inference_subject_family_new() {
        let methods = InferenceSubjectFamilyMethods {
            get_name_text: |_| None,
            get_default_certainty: |_| 0,
            new_permission_granted: |_, _| {},
            make_adj_const_domain: |_, _, _| {},
            complete_model: |_| {},
            check_model: |_| {},
        };
        let family = InferenceSubjectFamily::new("test_family", methods);
        assert_eq!(family.name, "test_family");
    }

    #[test]
    fn test_is_within_follows_hierarchy() {
        // Create a hierarchy: root (idx 0) -> parent (idx 1) -> child (idx 2)
        let mut registry = vec![
            InferenceSubject::new(0, None, None, Some("root")),
            InferenceSubject::new(0, Some(0), None, Some("parent")),
            InferenceSubject::new(0, Some(1), None, Some("child")),
        ];

        // Child is within parent
        assert!(registry[2].is_within(&registry[1], &registry));
        // Child is within root
        assert!(registry[2].is_within(&registry[0], &registry));
        // Parent is within root
        assert!(registry[1].is_within(&registry[0], &registry));
        // Root is not within child
        assert!(!registry[0].is_within(&registry[2], &registry));
        // Root is not within parent
        assert!(!registry[0].is_within(&registry[1], &registry));
        // Sibling is not within sibling
        registry.push(InferenceSubject::new(0, Some(0), None, Some("sibling")));
        assert!(!registry[3].is_within(&registry[1], &registry));
    }

    #[test]
    fn test_is_strictly_within_excludes_self() {
        let registry = vec![
            InferenceSubject::new(0, None, None, Some("root")),
            InferenceSubject::new(0, Some(0), None, Some("child")),
        ];
        // Child is strictly within root
        assert!(registry[1].is_strictly_within(&registry[0], &registry));
        // Child is not strictly within itself
        assert!(!registry[1].is_strictly_within(&registry[1], &registry));
    }
}

