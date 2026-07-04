use std::fmt::{self, Display, Formatter};
use std::ptr;

use crate::kind_constructors::{KindConstructor, Variance};

/// Maximum arity for kind constructions (matching MAX_KIND_CONSTRUCTION_ARITY = 2).
///
/// Corresponds to `MAX_KIND_CONSTRUCTION_ARITY` in the C reference
/// (`services/kinds-module/Chapter 2/Kinds.w`).
pub const MAX_KIND_CONSTRUCTION_ARITY: usize = 2;

/// A kind — a tree structure representing a type in Inform 7.
///
/// Corresponds to `kind` in the C reference
/// (`services/kinds-module/Chapter 2/Kinds.w`, lines 9-14).
///
/// Kinds are trees built by applying kind constructors to other kinds.
/// A base kind like `number` is a leaf. A composite kind like
/// `list of numbers` is a unary construction. A binary construction
/// like `relation of numbers to texts` has two children.
/// Mutable per-kind-instance state that was previously stored on the static
/// KindConstructor and mutated via unsafe pointer casts.
///
/// These fields are per-kind-instance state in the C reference, but were
/// conflated with the immutable constructor metadata in the Rust port.
#[derive(Clone, Debug)]
pub struct KindMutableState {
    /// Range number for indexing.
    pub class_number: i32,
    /// Whether this is an arithmetic kind.
    pub arithmetic: bool,
    /// Whether this is an enumeration kind.
    pub enumeration: bool,
    /// Next free value index for enumeration kinds.
    pub next_free_value: i32,
    /// Where the superkind was set.
    pub superkind_set_at: Option<usize>,
    /// Whether the dimensional form is fixed (derived kind).
    pub dimensional_form_fixed: bool,
    /// Documentation reference.
    pub documentation_reference: Option<&'static str>,
    /// Specification text for the index.
    pub specification_text: Option<&'static str>,
}

impl KindMutableState {
    /// Create mutable state initialized from a KindConstructor's initial values.
    pub fn from_constructor(con: &KindConstructor) -> Self {
        KindMutableState {
            class_number: con.class_number,
            arithmetic: con.arithmetic,
            enumeration: con.enumeration,
            next_free_value: con.next_free_value,
            superkind_set_at: con.superkind_set_at,
            dimensional_form_fixed: con.dimensional_form_fixed,
            documentation_reference: con.documentation_reference,
            specification_text: con.specification_text,
        }
    }
}



#[derive(Clone, Debug)]
pub struct Kind {
    /// The constructor used to build this kind (never None for valid kinds).
    pub construct: &'static KindConstructor,
    /// Kind variable number (1-26 for A-Z), only meaningful when construct
    /// is CON_KIND_VARIABLE.
    pub kind_variable_number: u8,
    /// Child kinds (up to MAX_KIND_CONSTRUCTION_ARITY).
    pub kc_args: [Option<Box<Kind>>; MAX_KIND_CONSTRUCTION_ARITY],
    /// Index of this kind's constructor in a constructor registry.
    ///
    /// Used by `KindSubjects::to_kind` to look up the constructor without
    /// unsafe lifetime transmutation. Set to `usize::MAX` when the
    /// constructor is a `&'static` reference (the normal case).
    pub construct_id: usize,
    /// Mutable per-kind-instance state.
    pub mutable_state: KindMutableState,
}

impl Kind {
    /// Create a base kind (arity 0) for the given constructor.
    ///
    /// For arity 1 constructors, defaults the argument to `K_value`.
    /// For arity 2, defaults both to `K_value`.
    ///
    /// Corresponds to `Kinds::base_construction` in the C reference
    /// (`services/kinds-module/Chapter 2/Kinds.w`, lines 77-95).
    pub fn base_construction(con: &'static KindConstructor) -> Self {
        let mut kc_args: [Option<Box<Kind>>; MAX_KIND_CONSTRUCTION_ARITY] = [None, None];

        for kc_arg in kc_args.iter_mut().take(con.arity as usize) {
            *kc_arg = Some(Box::new(Kind::clone(&crate::familiar_kinds::K_value)));
        }

        Kind {
            construct: con,
            kind_variable_number: 0,
            kc_args,
            construct_id: usize::MAX,
            mutable_state: KindMutableState::from_constructor(con),
        }
    }

    /// Create a unary construction (arity 1) like `list of X`.
    ///
    /// Corresponds to `Kinds::unary_con` in the C reference
    /// (`services/kinds-module/Chapter 2/Kinds.w`, lines 143-150).
    pub fn unary_con(con: &'static KindConstructor, x: Kind) -> Self {
        assert_eq!(con.arity, 1, "unary_con requires arity 1 constructor");
        Kind {
            construct: con,
            kind_variable_number: 0,
            kc_args: [Some(Box::new(x)), None],
            construct_id: usize::MAX,
            mutable_state: KindMutableState::from_constructor(con),
        }
    }

    /// Create a binary construction (arity 2) like `relation of X to Y`.
    ///
    /// Corresponds to `Kinds::binary_con` in the C reference
    /// (`services/kinds-module/Chapter 2/Kinds.w`, lines 152-163).
    pub fn binary_con(con: &'static KindConstructor, x: Kind, y: Kind) -> Self {
        Kind {
            construct: con,
            kind_variable_number: 0,
            kc_args: [Some(Box::new(x)), Some(Box::new(y))],
            construct_id: usize::MAX,
            mutable_state: KindMutableState::from_constructor(con),
        }
    }

    /// Create a function kind using `CON_TUPLE_ENTRY` punctuation.
    ///
    /// A function kind like `phrase (number, text) -> truth state` is
    /// represented as a nested CON_TUPLE_ENTRY structure:
    ///
    /// ```text
    /// CON_TUPLE_ENTRY
    ///   ├── CON_TUPLE_ENTRY
    ///   │   ├── K_number
    ///   │   └── K_text
    ///   └── K_truth_state
    /// ```
    ///
    /// Corresponds to `Kinds::function_kind` in the C reference
    /// (`services/kinds-module/Chapter 2/Kinds.w`, lines 198-204).
    pub fn function_kind(args: &[Kind], return_k: Option<Kind>) -> Self {
        let ret = return_k.unwrap_or_else(|| Kind::clone(&crate::familiar_kinds::K_void));
        let mut current = ret;

        for arg in args.iter().rev() {
            current = Kind::binary_con(
                &crate::familiar_kinds::CON_TUPLE_ENTRY,
                arg.clone(),
                current,
            );
        }

        current
    }

    /// Create a kind variable placeholder (A-Z).
    ///
    /// `n` is the variable number (1-26 for A-Z). `declaration` is an
    /// optional constraint on what kinds this variable can represent.
    ///
    /// Corresponds to `Kinds::var_construction` in the C reference
    /// (`services/kinds-module/Chapter 2/Kinds.w`, lines 122-130).
    pub fn var_construction(n: u8, declaration: Option<Kind>) -> Self {
        assert!(
            (1..=26).contains(&n),
            "kind variable number must be 1-26"
        );
        Kind {
            construct: &crate::familiar_kinds::CON_KIND_VARIABLE,
            kind_variable_number: n,
            kc_args: [declaration.map(Box::new), None],
            construct_id: usize::MAX,
            mutable_state: KindMutableState::from_constructor(
                &crate::familiar_kinds::CON_KIND_VARIABLE,
            ),
        }
    }

    /// Returns the constructor used to build this kind.
    pub fn get_construct(&self) -> &'static KindConstructor {
        self.construct
    }

    /// Returns the arity of this kind's constructor.
    pub fn arity(&self) -> u8 {
        self.construct.arity
    }

    /// Returns true if this is a proper kind (arity > 0).
    pub fn is_proper(&self) -> bool {
        self.construct.arity > 0
    }

    /// Returns true if this is an intermediate kind (CON_INTERMEDIATE).
    pub fn is_intermediate(&self) -> bool {
        ptr::eq(self.construct, &*crate::familiar_kinds::CON_INTERMEDIATE)
    }

    /// Returns the kind variable number (0 if not a variable).
    pub fn get_variable_number(&self) -> u8 {
        self.kind_variable_number
    }

    /// Returns the single child for unary constructions.
    pub fn unary_material(&self) -> Option<&Kind> {
        self.kc_args[0].as_deref()
    }

    /// Returns the two children for binary constructions.
    pub fn binary_material(&self) -> (Option<&Kind>, Option<&Kind>) {
        (self.kc_args[0].as_deref(), self.kc_args[1].as_deref())
    }

    #[allow(clippy::should_implement_trait)]
    /// Structural equality for kinds.
    ///
    /// Compares constructors (by pointer identity), variable numbers,
    /// and all child kinds recursively.
    ///
    /// Corresponds to `Kinds::eq` in the C reference
    /// (`services/kinds-module/Chapter 2/Kinds.w`, lines 579-594).
    pub fn eq(&self, other: &Kind) -> bool {
        self == other
    }
}


impl PartialEq for Kind {
    fn eq(&self, other: &Self) -> bool {
        if !ptr::eq(self.construct, other.construct) {
            return false;
        }
        if self.kind_variable_number != other.kind_variable_number {
            return false;
        }
        for (a, b) in self.kc_args.iter().zip(other.kc_args.iter()) {
            match (a, b) {
                (Some(a), Some(b)) => {
                    if a != b {
                        return false;
                    }
                }
                (None, None) => {}
                _ => {
                    return false;
                }
            }
        }
        true
    }
}

impl Kind {
    /// Test if this kind conforms to `other` (is-a relationship).
    ///
    /// Returns true if `self` conforms to `other`. Walks the superkind
    /// chain of `self` looking for `other`. For proper constructors
    /// (arity > 0) with the same constructor, checks variance of each
    /// argument.
    ///
    /// Corresponds to `Kinds::conforms_to` in the C reference
    /// (`services/kinds-module/Chapter 2/Kinds.w`, lines 608-612).
    pub fn conforms_to(&self, other: &Kind) -> bool {
        use crate::familiar_kinds::K_nil;

        if self.eq(other) {
            return true;
        }
        if other.eq(&K_nil) {
            return true;
        }
        if self.eq(&K_nil) {
            return true;
        }

        // For proper constructors with the same constructor, check variance
        if self.arity() > 0
            && self.arity() == other.arity()
            && ptr::eq(self.construct, other.construct)
        {
            let mut all_conform = true;
            for i in 0..self.arity() as usize {
                match (&self.kc_args[i], &other.kc_args[i]) {
                    (Some(a), Some(b)) => match self.construct.variance[i] {
                        Variance::Covariant => {
                            if !a.conforms_to(b) {
                                all_conform = false;
                            }
                        }
                        Variance::Contravariant => {
                            if !b.conforms_to(a) {
                                all_conform = false;
                            }
                        }
                    },
                    (None, None) => {}
                    _ => {
                        all_conform = false;
                    }
                }
            }
            if all_conform {
                return true;
            }
        }

        // Walk the superkind chain
        if let Some(super_k) = crate::lattice::superkind(self) {
            return super_k.conforms_to(other);
        }

        false
    }

    /// Three-valued compatibility check.
    ///
    /// Returns `Always` if `self` always conforms to `other`,
    /// `Sometimes` if they might be compatible (e.g., through kind variables),
    /// and `Never` if they can never be compatible.
    ///
    /// Corresponds to `Kinds::compatible` in the C reference
    /// (`services/kinds-module/Chapter 2/Kinds.w`, lines 614-625).
    pub fn compatible(&self, other: &Kind) -> Compatibility {
        use crate::familiar_kinds::{K_nil, K_unknown, K_void};

        // Nil, void, and unknown are compatible with everything
        if self.eq(&K_nil) || other.eq(&K_nil) {
            return Compatibility::Always;
        }
        if self.eq(&K_void) || other.eq(&K_void) {
            return Compatibility::Always;
        }
        if self.eq(&K_unknown) || other.eq(&K_unknown) {
            return Compatibility::Always;
        }

        // If one conforms to the other, they're always compatible
        if self.conforms_to(other) || other.conforms_to(self) {
            return Compatibility::Always;
        }

        // For now, without kind variables, unrelated kinds are never compatible.
        // The `Sometimes` case will be added when kind variables are supported.
        Compatibility::Never
    }
}

/// Three-valued compatibility result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compatibility {
    /// The kinds are always compatible (one conforms to the other).
    Always,
    /// The kinds might be compatible (e.g., through kind variables).
    Sometimes,
    /// The kinds can never be compatible.
    Never,
}

impl Display for Kind {
    /// Pretty-print a kind.
    ///
    /// Base kinds: print the constructor name.
    /// Unary constructions: `"<name> of <child>"` (e.g., "list of numbers").
    /// Binary constructions: `"<name> of <child1> to <child2>"` (e.g.,
    /// "relation of numbers to texts").
    /// Kind variables: print the letter (A-Z).
    ///
    /// Corresponds to `Kinds::Textual::write_inner` in the C reference
    /// (`services/kinds-module/Chapter 2/Describing Kinds.w`, lines 551-560).
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use crate::familiar_kinds::CON_KIND_VARIABLE;

        // Kind variables
        if ptr::eq(self.construct, &*CON_KIND_VARIABLE) {
            let letter = (b'A' + self.kind_variable_number - 1) as char;
            return write!(f, "{}", letter);
        }

        let name = self.construct.name;

        if self.arity() == 0 {
            return write!(f, "{}", name);
        }

        // For arity > 0, substitute K and L placeholders with child kinds
        let child1 = self.kc_args[0].as_deref();
        let child2 = self.kc_args[1].as_deref();

        let s = name
            .replace(
                "K",
                &child1
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "value".to_string()),
            )
            .replace(
                "L",
                &child2
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "value".to_string()),
            );

        write!(f, "{}", s)
    }
}

/// Parse a kind from its textual representation.
///
/// This is a simplified parser that handles:
/// - Single-word base kinds: "number", "text", "thing", "value", etc.
/// - Unary constructions: "list of <kind>", "description of <kind>", etc.
/// - Binary constructions: "relation of <kind> to <kind>",
///   "phrase <kind> -> <kind>", etc.
///
/// This is a simplified version of the `<k-kind>` Preform grammar from
/// `services/kinds-module/Chapter 2/Describing Kinds.w`.
impl std::str::FromStr for Kind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::familiar_kinds::*;

        let s = s.trim();

        // Try single-word kinds first
        match s {
            "value" => return Ok(K_value.clone()),
            "stored value" => return Ok(K_stored_value.clone()),
            "sayable value" => return Ok(K_sayable_value.clone()),
            "understandable value" => return Ok(K_understandable_value.clone()),
            "arithmetic value" => return Ok(K_arithmetic_value.clone()),
            "real arithmetic value" => return Ok(K_real_arithmetic_value.clone()),
            "enumerated value" => return Ok(K_enumerated_value.clone()),
            "pointer value" => return Ok(K_pointer_value.clone()),
            "void" => return Ok(K_void.clone()),
            "nil" => return Ok(K_nil.clone()),
            "unknown" => return Ok(K_unknown.clone()),
            "number" => return Ok(K_number.clone()),
            "text" => return Ok(K_text.clone()),
            "object" => return Ok(K_object.clone()),
            "real number" => return Ok(K_real_number.clone()),
            "truth state" => return Ok(K_truth_state.clone()),
            "table" => return Ok(K_table.clone()),
            "unicode character" => return Ok(K_unicode_character.clone()),
            "verb" => return Ok(K_verb.clone()),
            _ => {}
        }

        // Try unary constructions: "list of <kind>", "description of <kind>", etc.
        if let Some(rest) = s.strip_prefix("list of ") {
            let child: Kind = rest.parse()?;
            return Ok(Kind::unary_con(&CON_list_of, child));
        }
        if let Some(rest) = s.strip_prefix("description of ") {
            let child: Kind = rest.parse()?;
            return Ok(Kind::unary_con(&CON_description, child));
        }
        if let Some(rest) = s.strip_prefix("property of ") {
            let child: Kind = rest.parse()?;
            return Ok(Kind::unary_con(&CON_property, child));
        }
        if let Some(rest) = s.strip_prefix("combination of ") {
            let child: Kind = rest.parse()?;
            return Ok(Kind::unary_con(&CON_combination, child));
        }
        if let Some(rest) = s.strip_prefix("variable of ") {
            let child: Kind = rest.parse()?;
            return Ok(Kind::unary_con(&CON_variable, child));
        }

        // Try binary constructions: "relation of <kind> to <kind>"
        if let Some(rest) = s.strip_prefix("relation of ") {
            if let Some((left, right)) = rest.split_once(" to ") {
                let child1: Kind = left.parse()?;
                let child2: Kind = right.parse()?;
                return Ok(Kind::binary_con(&CON_relation, child1, child2));
            }
        }

        // Try "phrase <kind> -> <kind>"
        if let Some(rest) = s.strip_prefix("phrase ") {
            if let Some((left, right)) = rest.split_once(" -> ") {
                let child1: Kind = left.parse()?;
                let child2: Kind = right.parse()?;
                return Ok(Kind::binary_con(&CON_phrase, child1, child2));
            }
        }

        Err(format!("unknown kind description: '{}'", s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::familiar_kinds::*;

    #[test]
    fn base_construction_creates_kind_with_correct_constructor() {
        let k = Kind::base_construction(&CON_NUMBER);
        assert!(ptr::eq(k.get_construct(), &*CON_NUMBER));
        assert_eq!(k.arity(), 0);
        assert!(k.kc_args[0].is_none());
        assert!(k.kc_args[1].is_none());
    }

    #[test]
    fn unary_con_creates_kind_with_one_child() {
        let k = Kind::unary_con(&CON_list_of, K_number.clone());
        assert!(ptr::eq(k.get_construct(), &*CON_list_of));
        assert_eq!(k.arity(), 1);
        assert!(k.kc_args[0].is_some());
        assert!(k.kc_args[1].is_none());
        assert!(k.unary_material().unwrap().eq(&K_number));
    }

    #[test]
    fn binary_con_creates_kind_with_two_children() {
        let k = Kind::binary_con(&CON_relation, K_number.clone(), K_text.clone());
        assert!(ptr::eq(k.get_construct(), &*CON_relation));
        assert_eq!(k.arity(), 2);
        assert!(k.kc_args[0].is_some());
        assert!(k.kc_args[1].is_some());
        let (c1, c2) = k.binary_material();
        assert!(c1.unwrap().eq(&K_number));
        assert!(c2.unwrap().eq(&K_text));
    }

    #[test]
    fn function_kind_creates_tuple_entry_structure() {
        let k = Kind::function_kind(&[K_number.clone(), K_text.clone()], Some(K_truth_state.clone()));
        // Should be a nested CON_TUPLE_ENTRY structure
        assert!(ptr::eq(k.get_construct(), &*CON_TUPLE_ENTRY));
        assert_eq!(k.arity(), 2);
    }

    #[test]
    fn var_construction_creates_kind_with_correct_variable_number() {
        let k = Kind::var_construction(1, None);
        assert!(ptr::eq(k.get_construct(), &*CON_KIND_VARIABLE));
        assert_eq!(k.get_variable_number(), 1);
    }

    #[test]
    fn var_construction_with_declaration() {
        let k = Kind::var_construction(5, Some(Kind::clone(&K_number)));
        assert_eq!(k.get_variable_number(), 5);
        assert!(k.kc_args[0].is_some());
        assert!(k.kc_args[0].as_deref().unwrap().eq(&K_number));
    }
    #[test]
    fn eq_equal_kinds() {
        let k1 = Kind::unary_con(&CON_list_of, K_number.clone());
        let k2 = Kind::unary_con(&CON_list_of, K_number.clone());
        assert!(k1.eq(&k2));
    }

    #[test]
    fn eq_different_constructors() {
        let k1 = Kind::unary_con(&CON_list_of, K_number.clone());
        let k2 = Kind::unary_con(&CON_description, K_number.clone());
        assert!(!k1.eq(&k2));
    }

    #[test]
    fn eq_different_children() {
        let k1 = Kind::unary_con(&CON_list_of, K_number.clone());
        let k2 = Kind::unary_con(&CON_list_of, K_text.clone());
        assert!(!k1.eq(&k2));
    }

    #[test]
    fn eq_same_structure_different_instances() {
        // Two separately constructed "list of numbers" should be equal
        let k1 = Kind::unary_con(&CON_list_of, K_number.clone());
        let k2 = Kind::unary_con(&CON_list_of, K_number.clone());
        assert!(k1.eq(&k2));
    }

    #[test]
    fn display_base_kind() {
        assert_eq!(format!("{}", &*K_number), "number");
        assert_eq!(format!("{}", &*K_text), "text");
        assert_eq!(format!("{}", &*K_object), "object");
    }

    #[test]
    fn display_unary_construction() {
        let k = Kind::unary_con(&CON_list_of, K_number.clone());
        assert_eq!(format!("{}", k), "list of number");
    }

    #[test]
    fn display_binary_construction() {
        let k = Kind::binary_con(&CON_relation, K_number.clone(), K_text.clone());
        assert_eq!(format!("{}", k), "relation of number to text");
    }

    #[test]
    fn display_kind_variable() {
        let k = Kind::var_construction(1, None);
        assert_eq!(format!("{}", k), "A");
        let k2 = Kind::var_construction(26, None);
        assert_eq!(format!("{}", k2), "Z");
    }

    #[test]
    fn display_protocol_kind() {
        assert_eq!(format!("{}", &*K_value), "value");
        assert_eq!(format!("{}", &*K_arithmetic_value), "arithmetic value");
    }

    #[test]
    fn is_proper_returns_true_for_arity_positive() {
        let k = Kind::unary_con(&CON_list_of, K_number.clone());
        assert!(k.is_proper());
    }

    #[test]
    fn is_proper_returns_false_for_base_kind() {
        assert!(!K_number.is_proper());
    }

    #[test]
    fn is_intermediate_returns_false_for_normal_kind() {
        assert!(!K_number.is_intermediate());
    }

    #[test]
    fn get_construct_returns_correct_constructor() {
        assert!(ptr::eq(K_number.get_construct(), &*CON_NUMBER));
    }

    #[test]
    fn arity_returns_correct_value() {
        assert_eq!(K_number.arity(), 0);
        let list_of_num = Kind::unary_con(&CON_list_of, K_number.clone());
        assert_eq!(list_of_num.arity(), 1);
        let rel = Kind::binary_con(&CON_relation, K_number.clone(), K_text.clone());
        assert_eq!(rel.arity(), 2);
    }

    #[test]
    fn unary_material_returns_child() {
        let k = Kind::unary_con(&CON_list_of, K_number.clone());
        assert!(k.unary_material().unwrap().eq(&K_number));
    }

    #[test]
    fn unary_material_returns_none_for_base_kind() {
        assert!(K_number.unary_material().is_none());
    }

    #[test]
    fn binary_material_returns_children() {
        let k = Kind::binary_con(&CON_relation, K_number.clone(), K_text.clone());
        let (c1, c2) = k.binary_material();
        assert!(c1.unwrap().eq(&K_number));
        assert!(c2.unwrap().eq(&K_text));
    }

    #[test]
    fn from_str_parses_base_kinds() {
        use std::str::FromStr;
        assert!(Kind::from_str("number").unwrap().eq(&K_number));
        assert!(Kind::from_str("text").unwrap().eq(&K_text));
        assert!(Kind::from_str("object").unwrap().eq(&K_object));
        assert!(Kind::from_str("value").unwrap().eq(&K_value));
    }

    #[test]
    fn from_str_parses_unary_constructions() {
        use std::str::FromStr;
        let expected = Kind::unary_con(&CON_list_of, K_number.clone());
        let parsed = Kind::from_str("list of number").unwrap();
        assert!(parsed.eq(&expected));
    }

    #[test]
    fn from_str_parses_binary_constructions() {
        use std::str::FromStr;
        let expected = Kind::binary_con(&CON_relation, K_number.clone(), K_text.clone());
        let parsed = Kind::from_str("relation of number to text").unwrap();
        assert!(parsed.eq(&expected));
    }

    #[test]
    fn from_str_round_trip() {
        use std::str::FromStr;
        let kinds = [
            K_number.clone(),
            K_text.clone(),
            K_object.clone(),
            K_value.clone(),
            Kind::unary_con(&CON_list_of, K_number.clone()),
            Kind::binary_con(&CON_relation, K_number.clone(), K_text.clone()),
        ];
        for kind in &kinds {
            let s = format!("{}", kind);
            let parsed = Kind::from_str(&s).unwrap();
            assert!(parsed.eq(kind), "round-trip failed for '{}'", s);
        }
    }

    #[test]
    fn from_str_returns_error_for_unknown_kind() {
        use std::str::FromStr;
        let result = Kind::from_str("nonexistent");
        assert!(result.is_err());
    }
}
