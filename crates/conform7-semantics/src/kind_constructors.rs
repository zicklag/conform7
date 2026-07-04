/// Group classification for kind constructors.
///
/// Corresponds to the group field in `kind_constructor` in the C reference
/// (`services/kinds-module/Chapter 4/Kind Constructors.w`, lines 24-97).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConstructorGroup {
    /// Punctuation nodes used in kind construction only
    /// (CON_TUPLE_ENTRY, CON_VOID, CON_NIL, CON_UNKNOWN, CON_INTERMEDIATE, CON_KIND_VARIABLE).
    Punctuation,
    /// Protocol-like "kinds of kinds" (arithmetic value, sayable value, etc.).
    Protocol,
    /// Base constructors with arity 0 (number, text, thing, etc.).
    Base,
    /// Proper constructors with positive arity
    /// (list of ..., relation of ... to ..., etc.).
    Proper,
}

/// Variance of a constructor argument.
///
/// Corresponds to the variance field in `kind_constructor` in the C reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Variance {
    Covariant,
    Contravariant,
}

/// Tupling permission for a constructor argument.
///
/// Corresponds to the tupling field in `kind_constructor` in the C reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tupling {
    /// No tupling allowed.
    None,
    /// Allow nothing tupling (e.g., "nothing" as a list).
    AllowNothing,
    /// Arbitrary tupling allowed.
    Arbitrary,
}

/// A unit entry in a dimensional form (simplified).
///
/// Corresponds to the unit entries in the dimensional form of a quasinumerical kind
/// (`services/kinds-module/Chapter 4/Kind Constructors.w`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitEntry {
    /// Index into a unit registry (simplified).
    pub unit_kind: usize,
    /// Exponent for this unit.
    pub exponent: i32,
}

/// A unit sequence representing the dimensional form of a quasinumerical kind.
///
/// Corresponds to the dimensional form in `kind_constructor` in the C reference
/// (`services/kinds-module/Chapter 4/Kind Constructors.w`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitSequence {
    pub units: Vec<UnitEntry>,
}

impl UnitSequence {
    /// Create a scalar (dimensionless) unit sequence.
    pub fn scalar() -> Self {
        UnitSequence { units: Vec::new() }
    }

    /// Create a unit sequence from a list of unit entries.
    pub fn new(units: Vec<UnitEntry>) -> Self {
        UnitSequence { units }
    }
}

/// Dimensional operation type.
///
/// Corresponds to the dimensional operations in the C reference
/// (`services/kinds-module/Chapter 4/Kind Constructors.w`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DimensionalOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Root,
    Power,
}

/// A single dimensional rule for arithmetic operations on a kind.
///
/// Corresponds to the dimensional rules in `kind_constructor` in the C reference
/// (`services/kinds-module/Chapter 4/Kind Constructors.w`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DimensionalRule {
    pub operation: DimensionalOp,
    /// Simplified: kind index for the result kind.
    pub result_kind: Option<usize>,
}

/// Dimensional rules for arithmetic operations on a kind.
///
/// Corresponds to the dim_rules field in `kind_constructor` in the C reference
/// (`services/kinds-module/Chapter 4/Kind Constructors.w`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DimensionalRules {
    pub rules: Vec<DimensionalRule>,
}

impl DimensionalRules {
    /// Create empty dimensional rules.
    pub const fn new() -> Self {
        DimensionalRules { rules: Vec::new() }
    }

    /// Create dimensional rules from a list of rules.
    pub fn from_rules(rules: Vec<DimensionalRule>) -> Self {
        DimensionalRules { rules }
    }
}

impl Default for DimensionalRules {
    fn default() -> Self {
        DimensionalRules::new()
    }
}

/// A kind constructor — the "type constructor" that builds kinds.
///
/// Corresponds to `kind_constructor` in the C reference
/// (`services/kinds-module/Chapter 4/Kind Constructors.w`, lines 24-97).
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KindConstructor {
    /// The name of this constructor (e.g., "number", "list of K",
    /// "relation of K to L").
    pub name: &'static str,
    /// The group classification.
    pub group: ConstructorGroup,
    /// Arity: 0 for base, 1 for unary, 2 for binary.
    pub arity: u8,
    /// Variance of each argument (only meaningful for arity > 0).
    pub variance: [Variance; 2],
    /// Tupling of each argument.
    pub tupling: [Tupling; 2],
    /// Whether this is a definite kind (not a protocol/indefinite kind).
    pub definite: bool,
    /// Whether this is an arithmetic kind.
    pub arithmetic: bool,
    /// Whether this is an enumeration kind.
    pub enumeration: bool,
    /// Whether this is an object kind (subkind of object).
    pub object_kind: bool,

    // -----------------------------------------------------------------------
    // Behaviour API fields (added in PLAN-15)
    // Corresponds to `kind_constructor` fields in
    // `services/kinds-module/Chapter 4/Kind Constructors.w`.
    // -----------------------------------------------------------------------

    /// Whether this kind is incompletely defined (uncertainly defined).
    pub is_incompletely_defined: bool,

    /// Next free value index for enumeration kinds.
    pub next_free_value: i32,

    /// Explicit identifier for this kind (e.g., "K_number").
    pub explicit_identifier: Option<&'static str>,

    /// Where this kind was defined in source text (None for built-in kinds).
    pub where_defined_in_source_text: Option<usize>,

    /// How constants of this kind are compiled.
    pub constant_compilation_method: i32,

    /// Dimensional form for quasinumerical kinds.
    pub dimensional_form: Option<Box<UnitSequence>>,

    /// Whether the dimensional form is fixed (derived kind).
    pub dimensional_form_fixed: bool,

    /// Specification text for the index.
    pub specification_text: Option<&'static str>,

    /// Index priority.
    pub index_priority: i32,

    /// Whether to grey-out in index if empty.
    pub indexed_grey_if_empty: bool,

    /// Default value for the index.
    pub index_default_value: Option<&'static str>,

    /// Minimum value for the index.
    pub index_minimum_value: Option<&'static str>,

    /// Maximum value for the index.
    pub index_maximum_value: Option<&'static str>,

    /// Documentation reference.
    pub documentation_reference: Option<&'static str>,

    /// Whether values of this kind can be serialized.
    pub can_exchange: bool,

    /// Distinguishing routine name (None means ~= is sufficient).
    pub distinguishing_routine: Option<&'static str>,

    /// Whether this kind uses block (pointer) values.
    pub uses_block_values: bool,

    /// Small block size for block values.
    pub small_block_size: i32,

    /// Heap size estimate for block values.
    pub heap_size_estimate: i32,

    /// Range number for indexing.
    pub class_number: i32,

    /// Where the superkind was set.
    pub superkind_set_at: Option<usize>,

    /// Whether this kind uses signed comparisons.
    pub uses_signed_comparisons: bool,

    /// Comparison routine identifier.
    pub comparison_fn_identifier: Option<&'static str>,

    /// Dimensional rules for arithmetic operations.
    pub dim_rules: DimensionalRules,

    /// The inference subject for this base kind constructor (if any).
    ///
    /// Corresponds to `base_as_infs` in the C reference
    /// (`services/kinds-module/Chapter 4/Kind Constructors.w`).
    /// Only set for base kind constructors (arity 0, not CON_KIND_VARIABLE, not CON_INTERMEDIATE).
    pub base_as_infs: Option<usize>,
}

impl KindConstructor {
    /// Create a new kind constructor with default values.
    ///
    /// Defaults: covariant variance for all arguments, no tupling,
    /// not definite, not arithmetic, not enumeration, not object kind,
    /// all Behaviour fields set to false/0/None.
    pub const fn new(name: &'static str, group: ConstructorGroup, arity: u8) -> Self {
        KindConstructor {
            name,
            group,
            arity,
            variance: [Variance::Covariant, Variance::Covariant],
            tupling: [Tupling::None, Tupling::None],
            definite: false,
            arithmetic: false,
            enumeration: false,
            object_kind: false,
            is_incompletely_defined: false,
            next_free_value: 0,
            explicit_identifier: None,
            where_defined_in_source_text: None,
            constant_compilation_method: 0,
            dimensional_form: None,
            dimensional_form_fixed: false,
            specification_text: None,
            index_priority: 0,
            indexed_grey_if_empty: false,
            index_default_value: None,
            index_minimum_value: None,
            index_maximum_value: None,
            documentation_reference: None,
            can_exchange: false,
            distinguishing_routine: None,
            dim_rules: DimensionalRules::new(),
            base_as_infs: None,
            uses_block_values: false,
            small_block_size: 0,
            heap_size_estimate: 0,
            class_number: 0,
            superkind_set_at: None,
            uses_signed_comparisons: false,
            comparison_fn_identifier: None,
        }
    }

    /// Set the variance for a specific argument (0 or 1).
    pub fn set_variance(&mut self, arg: usize, variance: Variance) -> &mut Self {
        assert!(arg < 2, "argument index must be 0 or 1");
        self.variance[arg] = variance;
        self
    }

    /// Set the tupling for a specific argument (0 or 1).
    pub fn set_tupling(&mut self, arg: usize, tupling: Tupling) -> &mut Self {
        assert!(arg < 2, "argument index must be 0 or 1");
        self.tupling[arg] = tupling;
        self
    }

    /// Returns true if this is a definite kind (not a protocol/indefinite kind).
    pub fn is_definite(&self) -> bool {
        self.definite
    }

    /// Returns true if this is an arithmetic kind.
    pub fn is_arithmetic(&self) -> bool {
        self.arithmetic
    }

    /// Returns true if this is an enumeration kind.
    pub fn is_enumeration(&self) -> bool {
        self.enumeration
    }

    /// Returns true if this is an object kind (subkind of object).
    pub fn is_object_kind(&self) -> bool {
        self.object_kind
    }

    // -----------------------------------------------------------------------
    // Behaviour API setter methods
    // Corresponds to setter functions in
    // `services/kinds-module/Chapter 2/Using Kinds.w`.
    // -----------------------------------------------------------------------

    /// Set whether this kind is incompletely defined.
    pub fn set_incompletely_defined(&mut self, val: bool) -> &mut Self {
        self.is_incompletely_defined = val;
        self
    }

    /// Set the explicit identifier for this kind.
    pub fn set_explicit_identifier(&mut self, id: &'static str) -> &mut Self {
        self.explicit_identifier = Some(id);
        self
    }

    /// Set the constant compilation method.
    pub fn set_constant_compilation_method(&mut self, method: i32) -> &mut Self {
        self.constant_compilation_method = method;
        self
    }

    /// Set whether this kind uses block (pointer) values.
    pub fn set_uses_block_values(&mut self, val: bool) -> &mut Self {
        self.uses_block_values = val;
        self
    }

    /// Set the small block size for block values.
    pub fn set_small_block_size(&mut self, size: i32) -> &mut Self {
        self.small_block_size = size;
        self
    }

    /// Set the heap size estimate for block values.
    pub fn set_heap_size_estimate(&mut self, size: i32) -> &mut Self {
        self.heap_size_estimate = size;
        self
    }

    /// Set whether values of this kind can be serialized.
    pub fn set_can_exchange(&mut self, val: bool) -> &mut Self {
        self.can_exchange = val;
        self
    }

    /// Set whether this kind uses signed comparisons.
    pub fn set_uses_signed_comparisons(&mut self, val: bool) -> &mut Self {
        self.uses_signed_comparisons = val;
        self
    }

    /// Set the comparison routine identifier.
    pub fn set_comparison_fn_identifier(&mut self, id: &'static str) -> &mut Self {
        self.comparison_fn_identifier = Some(id);
        self
    }

    /// Set the distinguishing routine name.
    pub fn set_distinguishing_routine(&mut self, routine: &'static str) -> &mut Self {
        self.distinguishing_routine = Some(routine);
        self
    }

    /// Set the documentation reference.
    pub fn set_documentation_reference(&mut self, dr: &'static str) -> &mut Self {
        self.documentation_reference = Some(dr);
        self
    }

    /// Set the index priority.
    pub fn set_index_priority(&mut self, priority: i32) -> &mut Self {
        self.index_priority = priority;
        self
    }

    /// Set whether to grey-out in index if empty.
    pub fn set_indexed_grey_if_empty(&mut self, val: bool) -> &mut Self {
        self.indexed_grey_if_empty = val;
        self
    }

    /// Set the index default value.
    pub fn set_index_default_value(&mut self, val: &'static str) -> &mut Self {
        self.index_default_value = Some(val);
        self
    }

    /// Set the index minimum value.
    pub fn set_index_minimum_value(&mut self, val: &'static str) -> &mut Self {
        self.index_minimum_value = Some(val);
        self
    }

    /// Set the index maximum value.
    pub fn set_index_maximum_value(&mut self, val: &'static str) -> &mut Self {
        self.index_maximum_value = Some(val);
        self
    }

    /// Set the specification text.
    pub fn set_specification_text(&mut self, text: &'static str) -> &mut Self {
        self.specification_text = Some(text);
        self
    }

    /// Set the class number.
    pub fn set_class_number(&mut self, n: i32) -> &mut Self {
        self.class_number = n;
        self
    }

    /// Set the dimensional form.
    pub fn set_dimensional_form(&mut self, form: UnitSequence) -> &mut Self {
        self.dimensional_form = Some(Box::new(form));
        self
    }

    /// Set whether the dimensional form is fixed.
    pub fn set_dimensional_form_fixed(&mut self, val: bool) -> &mut Self {
        self.dimensional_form_fixed = val;
        self
    }

    /// Set the dimensional rules.
    pub fn set_dim_rules(&mut self, rules: DimensionalRules) -> &mut Self {
        self.dim_rules = rules;
        self
    }

    /// Set the inference subject for this base kind constructor.
    ///
    /// Corresponds to setting `base_as_infs` in the C reference
    /// (`services/kinds-module/Chapter 4/Kind Constructors.w`).
    pub fn set_base_as_infs(&mut self, infs: usize) -> &mut Self {
        self.base_as_infs = Some(infs);
        self
    }

    /// Get the inference subject for this base kind constructor, if any.
    ///
    /// Corresponds to reading `base_as_infs` in the C reference
    /// (`services/kinds-module/Chapter 4/Kind Constructors.w`).
    pub fn get_base_as_infs(&self) -> Option<usize> {
        self.base_as_infs
    }
}

/// Create the built-in kind constructors.
///
/// Corresponds to `Task::make_built_in_kind_constructors` in the C reference
/// (`inform7/core-module/Chapter 1/What To Compile.w`, lines 226-229).
///
/// Simplified: creates the constructors directly instead of loading from
/// project data.
///
/// Returns the indices of the created constructors in order:
/// [nil, tuple_entry, intermediate, kind_variable, rval_function,
///  list_of, description_of, table_of, combination, unchecked, named]
pub fn make_built_in(constructors: &mut Vec<KindConstructor>) -> [usize; 11] {
    let nil = { constructors.push(KindConstructor::new("nil", ConstructorGroup::Punctuation, 0)); constructors.len() - 1 };
    let tuple_entry = { constructors.push(KindConstructor::new("tuple_entry", ConstructorGroup::Punctuation, 0)); constructors.len() - 1 };
    let intermediate = { constructors.push(KindConstructor::new("intermediate", ConstructorGroup::Punctuation, 0)); constructors.len() - 1 };
    let kind_variable = { constructors.push(KindConstructor::new("kind_variable", ConstructorGroup::Punctuation, 0)); constructors.len() - 1 };
    let rval_function = { constructors.push(KindConstructor::new("rval_function", ConstructorGroup::Punctuation, 0)); constructors.len() - 1 };
    let list_of = { constructors.push(KindConstructor::new("list_of", ConstructorGroup::Proper, 1)); constructors.len() - 1 };
    let description_of = { constructors.push(KindConstructor::new("description_of", ConstructorGroup::Proper, 1)); constructors.len() - 1 };
    let table_of = { constructors.push(KindConstructor::new("table_of", ConstructorGroup::Proper, 1)); constructors.len() - 1 };
    let combination = { constructors.push(KindConstructor::new("combination", ConstructorGroup::Proper, 0)); constructors.len() - 1 };
    let unchecked = { constructors.push(KindConstructor::new("unchecked", ConstructorGroup::Punctuation, 0)); constructors.len() - 1 };
    let named = { constructors.push(KindConstructor::new("named", ConstructorGroup::Punctuation, 0)); constructors.len() - 1 };
    [nil, tuple_entry, intermediate, kind_variable, rval_function,
     list_of, description_of, table_of, combination, unchecked, named]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_constructor_has_arity_zero_and_base_group() {
        let con = KindConstructor::new("number", ConstructorGroup::Base, 0);
        assert_eq!(con.arity, 0);
        assert_eq!(con.group, ConstructorGroup::Base);
    }

    #[test]
    fn proper_constructor_has_arity_positive_and_proper_group() {
        let con = KindConstructor::new("list of K", ConstructorGroup::Proper, 1);
        assert_eq!(con.arity, 1);
        assert_eq!(con.group, ConstructorGroup::Proper);
    }

    #[test]
    fn protocol_constructor_has_protocol_group_and_is_not_definite() {
        let con = KindConstructor::new("value", ConstructorGroup::Protocol, 0);
        assert_eq!(con.group, ConstructorGroup::Protocol);
        assert!(!con.is_definite());
    }

    #[test]
    fn variance_setter_works_for_both_arguments() {
        let mut con = KindConstructor::new("phrase K -> L", ConstructorGroup::Proper, 2);
        con.set_variance(0, Variance::Contravariant);
        con.set_variance(1, Variance::Covariant);
        assert_eq!(con.variance[0], Variance::Contravariant);
        assert_eq!(con.variance[1], Variance::Covariant);
    }

    #[test]
    fn tupling_setter_works_for_both_arguments() {
        let mut con = KindConstructor::new("relation of K to L", ConstructorGroup::Proper, 2);
        con.set_tupling(0, Tupling::AllowNothing);
        con.set_tupling(1, Tupling::Arbitrary);
        assert_eq!(con.tupling[0], Tupling::AllowNothing);
        assert_eq!(con.tupling[1], Tupling::Arbitrary);
    }

    #[test]
    fn default_constructor_values() {
        let con = KindConstructor::new("test", ConstructorGroup::Base, 0);
        assert_eq!(con.variance, [Variance::Covariant, Variance::Covariant]);
        assert_eq!(con.tupling, [Tupling::None, Tupling::None]);
        assert!(!con.definite);
        assert!(!con.arithmetic);
        assert!(!con.enumeration);
        assert!(!con.object_kind);
    }

    #[test]
    fn flags_can_be_set() {
        let mut con = KindConstructor::new("number", ConstructorGroup::Base, 0);
        con.definite = true;
        con.arithmetic = true;
        assert!(con.is_definite());
        assert!(con.is_arithmetic());
        assert!(!con.is_enumeration());
        assert!(!con.is_object_kind());
    }

    #[test]
    fn new_behaviour_fields_default_to_false_zero_none() {
        let con = KindConstructor::new("test", ConstructorGroup::Base, 0);
        assert!(!con.is_incompletely_defined);
        assert_eq!(con.next_free_value, 0);
        assert!(con.explicit_identifier.is_none());
        assert!(con.where_defined_in_source_text.is_none());
        assert_eq!(con.constant_compilation_method, 0);
        assert!(con.dimensional_form.is_none());
        assert!(!con.dimensional_form_fixed);
        assert!(con.specification_text.is_none());
        assert_eq!(con.index_priority, 0);
        assert!(!con.indexed_grey_if_empty);
        assert!(con.index_default_value.is_none());
        assert!(con.index_minimum_value.is_none());
        assert!(con.index_maximum_value.is_none());
        assert!(con.documentation_reference.is_none());
        assert!(!con.can_exchange);
        assert!(con.distinguishing_routine.is_none());
        assert!(!con.uses_block_values);
        assert_eq!(con.small_block_size, 0);
        assert_eq!(con.heap_size_estimate, 0);
        assert_eq!(con.class_number, 0);
        assert!(con.superkind_set_at.is_none());
        assert!(!con.uses_signed_comparisons);
        assert!(con.comparison_fn_identifier.is_none());
        assert!(con.dim_rules.rules.is_empty());
    }

    #[test]
    fn setter_methods_update_fields_correctly() {
        let mut con = KindConstructor::new("test", ConstructorGroup::Base, 0);

        con.set_incompletely_defined(true);
        assert!(con.is_incompletely_defined);

        con.set_explicit_identifier("K_test");
        assert_eq!(con.explicit_identifier, Some("K_test"));

        con.set_constant_compilation_method(42);
        assert_eq!(con.constant_compilation_method, 42);

        con.set_uses_block_values(true);
        assert!(con.uses_block_values);

        con.set_small_block_size(64);
        assert_eq!(con.small_block_size, 64);

        con.set_heap_size_estimate(128);
        assert_eq!(con.heap_size_estimate, 128);

        con.set_can_exchange(true);
        assert!(con.can_exchange);

        con.set_uses_signed_comparisons(true);
        assert!(con.uses_signed_comparisons);

        con.set_comparison_fn_identifier("SignedCompare");
        assert_eq!(con.comparison_fn_identifier, Some("SignedCompare"));

        con.set_distinguishing_routine("TestDistinguisher");
        assert_eq!(con.distinguishing_routine, Some("TestDistinguisher"));

        con.set_documentation_reference("doc/test");
        assert_eq!(con.documentation_reference, Some("doc/test"));

        con.set_index_priority(5);
        assert_eq!(con.index_priority, 5);

        con.set_indexed_grey_if_empty(true);
        assert!(con.indexed_grey_if_empty);

        con.set_index_default_value("0");
        assert_eq!(con.index_default_value, Some("0"));

        con.set_index_minimum_value("1");
        assert_eq!(con.index_minimum_value, Some("1"));

        con.set_index_maximum_value("100");
        assert_eq!(con.index_maximum_value, Some("100"));

        con.set_specification_text("A test kind");
        assert_eq!(con.specification_text, Some("A test kind"));

        con.set_class_number(7);
        assert_eq!(con.class_number, 7);
    }

    #[test]
    fn unit_sequence_can_be_constructed() {
        let scalar = UnitSequence::scalar();
        assert!(scalar.units.is_empty());

        let entries = vec![
            UnitEntry { unit_kind: 0, exponent: 1 },
            UnitEntry { unit_kind: 1, exponent: -1 },
        ];
        let seq = UnitSequence::new(entries.clone());
        assert_eq!(seq.units.len(), 2);
        assert_eq!(seq.units[0].unit_kind, 0);
        assert_eq!(seq.units[0].exponent, 1);
        assert_eq!(seq.units[1].unit_kind, 1);
        assert_eq!(seq.units[1].exponent, -1);
    }

    #[test]
    fn dimensional_rules_can_be_constructed() {
        let empty = DimensionalRules::new();
        assert!(empty.rules.is_empty());

        let rules = vec![
            DimensionalRule {
                operation: DimensionalOp::Add,
                result_kind: Some(0),
            },
            DimensionalRule {
                operation: DimensionalOp::Multiply,
                result_kind: None,
            },
        ];
        let dr = DimensionalRules::from_rules(rules);
        assert_eq!(dr.rules.len(), 2);
        assert_eq!(dr.rules[0].operation, DimensionalOp::Add);
        assert_eq!(dr.rules[0].result_kind, Some(0));
        assert_eq!(dr.rules[1].operation, DimensionalOp::Multiply);
        assert_eq!(dr.rules[1].result_kind, None);
    }

    #[test]
    fn dimensional_form_setter_works() {
        let mut con = KindConstructor::new("number", ConstructorGroup::Base, 0);
        let form = UnitSequence::scalar();
        con.set_dimensional_form(form.clone());
        assert_eq!(con.dimensional_form, Some(Box::new(form)));

        con.set_dimensional_form_fixed(true);
        assert!(con.dimensional_form_fixed);
    }

    #[test]
    fn dim_rules_setter_works() {
        let mut con = KindConstructor::new("number", ConstructorGroup::Base, 0);
        let rules = DimensionalRules::from_rules(vec![DimensionalRule {
            operation: DimensionalOp::Add,
            result_kind: Some(0),
        }]);
        con.set_dim_rules(rules.clone());
        assert_eq!(con.dim_rules, rules);
    }

    #[test]
    fn base_as_infs_defaults_to_none() {
        let con = KindConstructor::new("number", ConstructorGroup::Base, 0);
        assert_eq!(con.base_as_infs, None);
        assert_eq!(con.get_base_as_infs(), None);
    }

    #[test]
    fn base_as_infs_setter_and_getter_work() {
        let mut con = KindConstructor::new("number", ConstructorGroup::Base, 0);
        con.set_base_as_infs(42);
        assert_eq!(con.base_as_infs, Some(42));
        assert_eq!(con.get_base_as_infs(), Some(42));

        // Overwrite
        con.set_base_as_infs(99);
        assert_eq!(con.get_base_as_infs(), Some(99));
    }

    #[test]
    fn make_built_in_creates_eleven_constructors() {
        let mut constructors = Vec::new();
        let indices = make_built_in(&mut constructors);
        assert_eq!(constructors.len(), 11);
        assert_eq!(indices.len(), 11);
    }

    #[test]
    fn make_built_in_constructor_names() {
        let mut constructors = Vec::new();
        let indices = make_built_in(&mut constructors);
        let expected_names = [
            "nil", "tuple_entry", "intermediate", "kind_variable",
            "rval_function", "list_of", "description_of", "table_of",
            "combination", "unchecked", "named",
        ];
        for (i, &idx) in indices.iter().enumerate() {
            assert_eq!(constructors[idx].name, expected_names[i],
                "constructor {} should have name '{}'", i, expected_names[i]);
        }
    }
}
