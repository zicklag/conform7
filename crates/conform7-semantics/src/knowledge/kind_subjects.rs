/// Kind Subjects — the bridge between the kind system and the knowledge module.
///
/// Every base kind gets its own inference subject, making it possible to
/// draw inferences from sentences such as:
/// "A scene has a number called the witness count. The witness count of a
/// scene is usually 4."
///
/// Corresponds to `KindSubjects` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`).
use crate::kind_constructors::KindConstructor;
use crate::kinds::Kind;
use crate::knowledge::inference_subjects::{
    InferenceSubject, InferenceSubjectFamily, InferenceSubjectFamilyMethods,
};

/// Map a constructor name to its superkind's constructor name.
///
/// This mirrors the superkind hierarchy defined in `lattice::superkind`
/// but works with constructor names rather than pointer comparisons,
/// making it suitable for use with cloned constructors.
fn superkind_name(con_name: &str) -> Option<&'static str> {
    match con_name {
        // Protocol kinds
        "value" => None,
        "stored value" => Some("value"),
        "sayable value" => Some("stored value"),
        "understandable value" => Some("sayable value"),
        "arithmetic value" => Some("understandable value"),
        "real arithmetic value" => Some("arithmetic value"),
        "enumerated value" => Some("understandable value"),
        "pointer value" => Some("sayable value"),
        // Base kinds
        "number" => Some("real number"),
        "real number" => Some("arithmetic value"),
        "text" => Some("sayable value"),
        "truth state" => Some("enumerated value"),
        "object" => Some("pointer value"),
        "table" => Some("stored value"),
        "unicode character" => Some("arithmetic value"),
        "verb" => Some("value"),
        // Proper constructors: superkind is value
        _ => Some("value"),
    }
}

/// The kinds family of inference subjects.
///
/// Every base kind gets its own inference subject, making it possible to
/// draw inferences about kinds.
///
/// Corresponds to `KindSubjects::family` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`, lines 96-110).
pub fn family() -> InferenceSubjectFamily {
    InferenceSubjectFamily {
        name: "kinds",
        methods: InferenceSubjectFamilyMethods {
            get_name_text: |subject| {
                // Return the log name as a stand-in for the kind's name.
                // In the C reference, this retrieves the kind constructor's name
                // from the subject's represents field.
                subject.log_name
            },
            get_default_certainty: |_| 1, // LIKELY_CE — inferences about kinds are likely, not certain
            new_permission_granted: |_, _| {
                // Stub: in the C reference, this allocates run-time storage
                // for the permission. That requires run-time compilation which
                // is out of scope for now.
            },
            make_adj_const_domain: |_, _, _| {
                // Stub: in the C reference, this registers an adjectival
                // constant for the kind. That requires instance adjectives
                // which are out of scope for now.
            },
            complete_model: |_| {
                // Stub: model completion is not yet implemented.
            },
            check_model: |_| {
                // Stub: model checking is not yet implemented.
            },
        },
    }
}

/// Create an inference subject for a base kind constructor.
///
/// Only creates subjects for base constructors (arity 0, not CON_KIND_VARIABLE,
/// not CON_INTERMEDIATE). Returns None for other constructors.
///
/// The subject is stored on the constructor via `set_base_as_infs` so that
/// `from_kind` can retrieve it later.
///
/// Corresponds to `KindSubjects::new` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`, lines 43-52).
pub fn new(
    con: &mut KindConstructor,
    subjects: &mut Vec<InferenceSubject>,
    families: &[InferenceSubjectFamily],
) -> Option<usize> {
    // Only create subjects for base constructors (arity 0) that are not
    // kind variables or intermediate results.
    if con.arity != 0 || con.name == "kind variable" || con.name == "intermediate" {
        return None;
    }

    // Find the kinds family index
    let family_idx = families.iter().position(|f| f.name == "kinds")?;

    // Determine the broader subject based on the superkind hierarchy.
    // Look up the superkind's constructor name and find its inference subject.
    let broader_than = superkind_name(con.name).and_then(|super_name| {
        subjects.iter().position(|s| s.represents == Some(super_name))
    });

    // Create the inference subject
    let subject = InferenceSubject::new(
        family_idx,
        broader_than,
        Some(con.name),
        Some(con.name),
    );

    let idx = subjects.len();
    subjects.push(subject);
    con.set_base_as_infs(idx);

    Some(idx)
}

/// Return the inference subject index for a kind.
///
/// Looks up the kind's constructor by name in the constructors slice and
/// returns the stored inference subject index (set by `KindSubjects::new`).
///
/// Corresponds to `KindSubjects::from_kind` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`, lines 54-56).
pub fn from_kind(kind: &Kind, constructors: &[KindConstructor]) -> Option<usize> {
    let con = constructors.iter().find(|c| c.name == kind.construct.name)?;
    con.base_as_infs
}

/// Return the kind for an inference subject.
///
/// Looks up the subject's `represents` field (which stores the constructor name)
/// and finds the matching constructor to build the kind.
///
/// Corresponds to `KindSubjects::to_kind` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`, lines 58-60).
#[allow(clippy::explicit_auto_deref)]
pub fn to_kind(
    infs: &InferenceSubject,
    _subjects: &[InferenceSubject],
    constructors: &[KindConstructor],
) -> Option<Kind> {
    let con_name = infs.represents?;
    let (idx, _) = constructors.iter().enumerate().find(|(_, c)| c.name == con_name)?;
    Some(Kind {
        construct: &*crate::familiar_kinds::CON_VALUE,
        kind_variable_number: 0,
        kc_args: [None, None],
        construct_id: idx,
    })
}

/// Test if a kind can have properties.
///
/// A kind can have properties if it is an enumeration kind or an object kind.
///
/// Corresponds to `KindSubjects::has_properties` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`, lines 62-66).
pub fn has_properties(kind: &Kind, constructors: &[KindConstructor]) -> bool {
    // First try to find the constructor in the constructors slice,
    // which may have been modified (e.g., enumeration set to true).
    if let Some(con) = constructors.iter().find(|c| c.name == kind.construct.name) {
        con.enumeration || con.object_kind
    } else {
        kind.construct.enumeration || kind.construct.object_kind
    }
}

// ---------------------------------------------------------------------------
// Lattice callbacks
//
// These callbacks connect the kind lattice to the subject hierarchy.
// They are called by the lattice operations (superkind, join, meet) to
// maintain the correspondence between the kind hierarchy and the subject
// hierarchy.
//
// Corresponds to the HIERARCHY_*_CALLBACK macros in the C reference
// (`services/kinds-module/Chapter 2/The Lattice of Kinds.w`, lines 89-119).
// ---------------------------------------------------------------------------

/// Callback: return the superkind's inference subject for a kind.
///
/// Follows the subject hierarchy upward from the kind's subject to find
/// the subject that represents the superkind.
///
/// Corresponds to `HIERARCHY_GET_SUPER_KINDS_CALLBACK` in the C reference
/// (`services/kinds-module/Chapter 2/The Lattice of Kinds.w`, lines 89-119).
pub fn super_callback(
    kind: &Kind,
    subjects: &[InferenceSubject],
    constructors: &[KindConstructor],
) -> Option<usize> {
    let infs_idx = from_kind(kind, constructors)?;
    let subject = &subjects[infs_idx];
    subject.narrowest_broader_subject()
}

/// Callback: move a kind within the subject hierarchy.
///
/// Updates the kind's inference subject to have the given superkind's
/// subject as its broader subject.
///
/// Corresponds to `HIERARCHY_MOVE_KINDS_CALLBACK` in the C reference
/// (`services/kinds-module/Chapter 2/The Lattice of Kinds.w`, lines 89-119).
pub fn move_within_callback(
    sub: &Kind,
    super_kind: &Kind,
    subjects: &mut [InferenceSubject],
    constructors: &[KindConstructor],
) -> Option<()> {
    let sub_infs = from_kind(sub, constructors)?;
    let super_infs = from_kind(super_kind, constructors)?;
    subjects[sub_infs].falls_within(super_infs);
    Some(())
}

/// Callback: check if a kind is within the object hierarchy.
///
/// Returns true if the kind's inference subject is within the object
/// kind's inference subject in the subject hierarchy.
///
/// Corresponds to `HIERARCHY_ALLOWS_SOMETIMES_MATCH_KINDS_CALLBACK` in the C reference
/// (`services/kinds-module/Chapter 2/The Lattice of Kinds.w`, lines 89-119).
pub fn allow_sometimes_callback(
    from: &Kind,
    subjects: &[InferenceSubject],
    constructors: &[KindConstructor],
) -> bool {
    let from_infs = match from_kind(from, constructors) {
        Some(idx) => idx,
        None => return false,
    };

    // Find the object kind's inference subject
    let object_con = match constructors.iter().find(|c| c.name == "object") {
        Some(con) => con,
        None => return false,
    };
    let object_infs = match object_con.base_as_infs {
        Some(idx) => idx,
        None => return false,
    };

    // Check if from's subject is within the object subject, or IS the object subject
    from_infs == object_infs || subjects[from_infs].is_within(&subjects[object_infs], subjects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::familiar_kinds::*;
    use crate::knowledge::setup::setup_knowledge_module;

    #[test]
    fn kinds_family_has_correct_name() {
        let f = family();
        assert_eq!(f.name, "kinds");
    }

    #[test]
    fn kinds_family_has_likely_certainty() {
        let f = family();
        let subject = InferenceSubject::new(0, None, None, Some("test"));
        assert_eq!((f.methods.get_default_certainty)(&subject), 1);
    }

    #[test]
    fn new_creates_subject_for_base_constructor() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let mut con = CON_NUMBER.clone();
        let idx = new(&mut con, &mut subjects, &families).unwrap();

        assert_eq!(con.base_as_infs, Some(idx));
        assert_eq!(subjects[idx].represents, Some("number"));
        assert_eq!(subjects[idx].log_name, Some("number"));
        assert_eq!(subjects[idx].infs_family, 1); // kinds family is at index 1
    }

    #[test]
    fn new_returns_none_for_kind_variable() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let mut con = CON_KIND_VARIABLE.clone();
        let result = new(&mut con, &mut subjects, &families);
        assert!(result.is_none());
    }

    #[test]
    fn new_returns_none_for_intermediate() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let mut con = CON_INTERMEDIATE.clone();
        let result = new(&mut con, &mut subjects, &families);
        assert!(result.is_none());
    }

    #[test]
    fn new_returns_none_for_proper_constructor() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let mut con = CON_list_of.clone();
        let result = new(&mut con, &mut subjects, &families);
        assert!(result.is_none());
    }

    #[test]
    fn from_kind_returns_subject_for_base_kind() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let mut con = CON_NUMBER.clone();
        new(&mut con, &mut subjects, &families);

        let constructors = vec![con];
        let k_number = Kind::base_construction(&CON_NUMBER);
        let infs_idx = from_kind(&k_number, &constructors).unwrap();
        assert_eq!(subjects[infs_idx].log_name, Some("number"));
    }

    #[test]
    fn from_kind_returns_none_for_kind_without_subject() {
        let constructors: Vec<KindConstructor> = vec![CON_list_of.clone()];
        let k_list = Kind::unary_con(&CON_list_of, Kind::base_construction(&CON_NUMBER));
        let result = from_kind(&k_list, &constructors);
        assert!(result.is_none());
    }
    #[test]
    fn to_kind_round_trips_with_from_kind() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let mut con = CON_NUMBER.clone();
        new(&mut con, &mut subjects, &families);

        let constructors = vec![con];
        let k_number = Kind::base_construction(&CON_NUMBER);
        let infs_idx = from_kind(&k_number, &constructors).unwrap();
        let infs = &subjects[infs_idx];
        let kind = to_kind(infs, &subjects, &constructors).unwrap();
        // Look up the constructor via construct_id instead of using
        // kind.construct directly (which is a dummy value for to_kind results)
        let con = &constructors[kind.construct_id];
        assert_eq!(con.name, k_number.construct.name);
        assert_eq!(con.arity, k_number.construct.arity);
        assert_eq!(kind.kind_variable_number, k_number.kind_variable_number);
    }

    #[test]
    fn to_kind_returns_none_for_subject_without_represents() {
        let subject = InferenceSubject::new(0, None, None, None);
        let constructors: Vec<KindConstructor> = vec![];
        let result = to_kind(&subject, &[], &constructors);
        assert!(result.is_none());
    }

    #[test]
    fn has_properties_true_for_enumeration() {
        let mut con = CON_NUMBER.clone();
        con.enumeration = true;
        let k = Kind::base_construction(&CON_NUMBER);
        let constructors = vec![con];
        assert!(has_properties(&k, &constructors));
    }

    #[test]
    fn has_properties_true_for_object_kind() {
        let k = Kind::base_construction(&CON_OBJECT);
        let constructors = vec![CON_OBJECT.clone()];
        assert!(has_properties(&k, &constructors));
    }

    #[test]
    fn has_properties_false_for_plain_base_kind() {
        let k = Kind::base_construction(&CON_NUMBER);
        let constructors = vec![CON_NUMBER.clone()];
        assert!(!has_properties(&k, &constructors));
    }

    #[test]
    fn super_callback_returns_broader_subject() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        // Create kind subjects for number and its superkind (real number)
        let mut con_real_number = CON_REAL_NUMBER.clone();
        new(&mut con_real_number, &mut subjects, &families);
        let real_number_infs = con_real_number.base_as_infs.unwrap();

        let mut con_number = CON_NUMBER.clone();
        new(&mut con_number, &mut subjects, &families);

        let constructors = vec![con_real_number, con_number];
        let k_number = Kind::base_construction(&CON_NUMBER);

        // The super callback should return the broader subject (real number's subject)
        let result = super_callback(&k_number, &subjects, &constructors);
        assert_eq!(result, Some(real_number_infs));
    }

    #[test]
    fn super_callback_returns_none_for_kind_without_subject() {
        let subjects: Vec<InferenceSubject> = vec![];
        let constructors: Vec<KindConstructor> = vec![CON_list_of.clone()];
        let k_list = Kind::unary_con(&CON_list_of, Kind::base_construction(&CON_NUMBER));
        let result = super_callback(&k_list, &subjects, &constructors);
        assert!(result.is_none());
    }

    #[test]
    fn move_within_callback_updates_subject_hierarchy() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let mut con_number = CON_NUMBER.clone();
        let mut con_text = CON_TEXT.clone();
        new(&mut con_number, &mut subjects, &families);
        new(&mut con_text, &mut subjects, &families);

        let constructors = vec![con_number, con_text];
        let k_number = Kind::base_construction(&CON_NUMBER);
        let k_text = Kind::base_construction(&CON_TEXT);

        // Move number under text
        let result = move_within_callback(&k_number, &k_text, &mut subjects, &constructors);
        assert!(result.is_some());

        // Verify the move
        let number_infs = from_kind(&k_number, &constructors).unwrap();
        let text_infs = from_kind(&k_text, &constructors).unwrap();
        assert_eq!(subjects[number_infs].broader_than, Some(text_infs));
    }

    #[test]
    fn move_within_callback_returns_none_for_kind_without_subject() {
        let mut subjects: Vec<InferenceSubject> = vec![];
        let constructors: Vec<KindConstructor> = vec![CON_list_of.clone()];
        let k_list = Kind::unary_con(&CON_list_of, Kind::base_construction(&CON_NUMBER));
        let k_number = Kind::base_construction(&CON_NUMBER);
        let result = move_within_callback(&k_list, &k_number, &mut subjects, &constructors);
        assert!(result.is_none());
    }

    #[test]
    fn allow_sometimes_callback_checks_object_hierarchy() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let mut con_object = CON_OBJECT.clone();
        let mut con_number = CON_NUMBER.clone();
        new(&mut con_object, &mut subjects, &families);
        new(&mut con_number, &mut subjects, &families);

        let constructors = vec![con_object, con_number];
        let k_object = Kind::base_construction(&CON_OBJECT);
        let k_number = Kind::base_construction(&CON_NUMBER);

        // number is not within the object hierarchy
        assert!(!allow_sometimes_callback(&k_number, &subjects, &constructors));

        // object is within the object hierarchy (it IS the object kind)
        assert!(allow_sometimes_callback(&k_object, &subjects, &constructors));
    }

    #[test]
    fn allow_sometimes_returns_false_for_kind_without_subject() {
        let subjects: Vec<InferenceSubject> = vec![];
        let constructors: Vec<KindConstructor> = vec![CON_list_of.clone()];
        let k_list = Kind::unary_con(&CON_list_of, Kind::base_construction(&CON_NUMBER));
        assert!(!allow_sometimes_callback(&k_list, &subjects, &constructors));
    }


    #[test]
    fn multiple_kind_subjects_can_be_created() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let mut con_number = CON_NUMBER.clone();
        let mut con_text = CON_TEXT.clone();
        let mut con_object = CON_OBJECT.clone();

        let idx1 = new(&mut con_number, &mut subjects, &families).unwrap();
        let idx2 = new(&mut con_text, &mut subjects, &families).unwrap();
        let idx3 = new(&mut con_object, &mut subjects, &families).unwrap();

        assert_eq!(idx1, 3); // After 3 fundamental subjects
        assert_eq!(idx2, 4);
        assert_eq!(idx3, 5);

        assert_eq!(subjects[idx1].log_name, Some("number"));
        assert_eq!(subjects[idx2].log_name, Some("text"));
        assert_eq!(subjects[idx3].log_name, Some("object"));
    }

    #[test]
    fn kind_subject_uses_kinds_family_methods() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let mut con = CON_NUMBER.clone();
        new(&mut con, &mut subjects, &families);

        // Test family method dispatch
        let subject = &subjects[con.base_as_infs.unwrap()];
        let name = subject.get_name_text(&families);
        assert_eq!(name, Some("number"));

        let certainty = subject.get_default_certainty(&families);
        assert_eq!(certainty, 1); // LIKELY_CE
    }
}
