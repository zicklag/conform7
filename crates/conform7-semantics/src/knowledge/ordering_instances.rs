/// The Ordering Instances system — a simple linked list for placing instances in order.
///
/// Corresponds to `OrderingInstances` in the C reference
/// (`inform7/knowledge-module/Chapter 2/Ordering Instances.w`).
///
/// Instances are stored as a linked list with links in an array indexed by
/// the instance's position in the instances vector. This ordering is used
/// not only for compilation, but also for instance counting (e.g., marking
/// the black gate as the 8th instance of "door"), so it's needed earlier
/// than the compilation phase.
///
/// Simplified:
/// - No `NUMBER_CREATED` macro (uses `instances.len()`)
/// - No `Memory::calloc` (uses `vec![None; instances.len()]`)
/// - No `OBJECT_COMPILATION_MREASON` memory reason
/// - No `LOOP_OVER_INSTANCES` macro (uses `instances.iter().enumerate()`)
/// - No `K_object` filtering (iterates all instances)
/// - No `allocation_id` field (uses vector index)
/// - No macros (uses methods and an iterator)
use std::mem::ManuallyDrop;
use crate::knowledge::instances::Instance;

/// The ordering instances module.
///
/// Corresponds to `OrderingInstances` in the C reference
/// (`inform7/knowledge-module/Chapter 2/Ordering Instances.w`).
pub struct OrderingInstances;

/// The first instance in the ordering, if any.
/// Corresponds to `first_instance_in_list` in the C reference.
static mut FIRST_INSTANCE: Option<usize> = None;

/// The last instance in the ordering, if any.
/// Corresponds to `last_instance_in_list` in the C reference.
static mut LAST_INSTANCE: Option<usize> = None;

/// The next-instance pointers, indexed by instance index.
/// Corresponds to `next_instance_in_current_list` in the C reference.
///
/// Wrapped in `ManuallyDrop` to prevent automatic drop at program exit,
/// which would cause a double-free on the heap-allocated Vec buffer.
static mut NEXT_INSTANCE: ManuallyDrop<Vec<Option<usize>>> = ManuallyDrop::new(Vec::new());

impl OrderingInstances {
    /// Initialise the ordering linked list.
    ///
    /// Corresponds to `OrderingInstances::begin` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Ordering Instances.w`, lines 23-32).
    ///
    /// Creates a next-instance array with one entry per instance, all set to None.
    /// Resets first and last instance pointers.
    pub fn begin(instances: &[Instance]) {
        unsafe {
            // Replace the Vec without dropping the old one (avoids double-free
            // on the heap-allocated Vec buffer at program exit).
            std::ptr::write(
                std::ptr::addr_of_mut!(NEXT_INSTANCE),
                ManuallyDrop::new(vec![None; instances.len()]),
            );
            FIRST_INSTANCE = None;
            LAST_INSTANCE = None;
        }
    }

    /// Add an instance to the end of the ordering list.
    ///
    /// Corresponds to `OrderingInstances::place_next` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Ordering Instances.w`, lines 34-40).
    ///
    /// If the list is empty, sets the first instance to this one.
    /// Otherwise, links the last instance to this one.
    /// Then sets the last instance to this one.
    pub fn place_next(instance_idx: usize) {
        unsafe {
            let last = std::ptr::addr_of!(LAST_INSTANCE).read();
            if let Some(last_idx) = last {
                NEXT_INSTANCE[last_idx] = Some(instance_idx);
            } else {
                FIRST_INSTANCE = Some(instance_idx);
            }
            LAST_INSTANCE = Some(instance_idx);
        }
    }

    /// Order all instances by definition sequence.
    ///
    /// Corresponds to `OrderingInstances::objects_in_definition_sequence` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Ordering Instances.w`, lines 45-50).
    ///
    /// Simplified: iterates over all instances (not just K_object instances).
    pub fn objects_in_definition_sequence(instances: &[Instance]) {
        Self::begin(instances);
        for i in 0..instances.len() {
            Self::place_next(i);
        }
    }

    /// Return the index of the first instance in the ordering.
    ///
    /// Corresponds to `FIRST_IN_INSTANCE_ORDERING` in the C reference.
    pub fn first() -> Option<usize> {
        unsafe { std::ptr::addr_of!(FIRST_INSTANCE).read() }
    }

    /// Return the index of the next instance after the given one in the ordering.
    ///
    /// Corresponds to `NEXT_IN_INSTANCE_ORDERING(I)` in the C reference.
    pub fn next(instance_idx: usize) -> Option<usize> {
        unsafe { (*NEXT_INSTANCE).get(instance_idx).copied().flatten() }
    }

    /// Return an iterator over the ordered instance indices.
    ///
    /// Corresponds to `LOOP_THROUGH_INSTANCE_ORDERING(I)` in the C reference.
    pub fn iter() -> OrderingIterator {
        OrderingIterator {
            current: Self::first(),
        }
    }
}

/// An iterator over ordered instance indices.
///
/// Corresponds to `LOOP_THROUGH_INSTANCE_ORDERING(I)` in the C reference.
pub struct OrderingIterator {
    current: Option<usize>,
}

impl Iterator for OrderingIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current;
        if let Some(idx) = self.current {
            self.current = OrderingInstances::next(idx);
        }
        result
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a minimal instance for testing.
    fn make_instance(name: &'static str) -> Instance {
        Instance {
            name,
            as_adjective: None,
            as_subject: None,
            enumeration_index: 0,
            kind_coincident: false,
        }
    }

    // -----------------------------------------------------------------------
    // OrderingInstances::begin
    // -----------------------------------------------------------------------

    #[test]
    fn test_begin_resets_list() {
        let instances = vec![make_instance("red"), make_instance("blue")];

        // Place some instances first.
        OrderingInstances::begin(&instances);
        OrderingInstances::place_next(0);
        OrderingInstances::place_next(1);

        // Now begin again — should reset.
        OrderingInstances::begin(&instances);

        assert_eq!(OrderingInstances::first(), None);
    }

    #[test]
    fn test_begin_creates_next_array_with_correct_size() {
        let instances = vec![
            make_instance("red"),
            make_instance("blue"),
            make_instance("green"),
        ];

        OrderingInstances::begin(&instances);

        // After begin, first and last should be None.
        assert_eq!(OrderingInstances::first(), None);

        // After placing, the next array should have the right size.
        OrderingInstances::place_next(0);
        OrderingInstances::place_next(1);
        OrderingInstances::place_next(2);

        // The last instance should have no next.
        assert_eq!(OrderingInstances::next(2), None);
    }

    // -----------------------------------------------------------------------
    // OrderingInstances::place_next
    // -----------------------------------------------------------------------

    #[test]
    fn test_place_next_adds_instances_in_order() {
        let instances = vec![make_instance("red"), make_instance("blue"), make_instance("green")];

        OrderingInstances::begin(&instances);
        OrderingInstances::place_next(0);
        OrderingInstances::place_next(1);
        OrderingInstances::place_next(2);

        // First should be the first placed.
        assert_eq!(OrderingInstances::first(), Some(0));

        // Next pointers should form a chain.
        assert_eq!(OrderingInstances::next(0), Some(1));
        assert_eq!(OrderingInstances::next(1), Some(2));
        assert_eq!(OrderingInstances::next(2), None);
    }

    #[test]
    fn test_place_next_first_call_sets_first_instance() {
        let instances = vec![make_instance("red")];

        OrderingInstances::begin(&instances);
        OrderingInstances::place_next(0);

        assert_eq!(OrderingInstances::first(), Some(0));
    }

    #[test]
    fn test_place_next_subsequent_calls_link_correctly() {
        let instances = vec![make_instance("red"), make_instance("blue"), make_instance("green")];

        OrderingInstances::begin(&instances);
        OrderingInstances::place_next(0);
        OrderingInstances::place_next(1);
        OrderingInstances::place_next(2);

        // Chain: 0 -> 1 -> 2 -> None
        assert_eq!(OrderingInstances::next(0), Some(1));
        assert_eq!(OrderingInstances::next(1), Some(2));
        assert_eq!(OrderingInstances::next(2), None);
    }

    #[test]
    fn test_place_next_last_call_sets_last_instance() {
        let instances = vec![make_instance("red"), make_instance("blue")];

        OrderingInstances::begin(&instances);
        OrderingInstances::place_next(0);
        OrderingInstances::place_next(1);

        // The last placed instance should have no next.
        assert_eq!(OrderingInstances::next(1), None);
    }

    // -----------------------------------------------------------------------
    // OrderingInstances::objects_in_definition_sequence
    // -----------------------------------------------------------------------

    #[test]
    fn test_objects_in_definition_sequence_orders_by_creation_order() {
        let instances = vec![
            make_instance("red"),
            make_instance("blue"),
            make_instance("green"),
        ];

        OrderingInstances::objects_in_definition_sequence(&instances);

        // Should be ordered by index: 0, 1, 2.
        assert_eq!(OrderingInstances::first(), Some(0));
        assert_eq!(OrderingInstances::next(0), Some(1));
        assert_eq!(OrderingInstances::next(1), Some(2));
        assert_eq!(OrderingInstances::next(2), None);
    }

    #[test]
    fn test_objects_in_definition_sequence_includes_all_instances() {
        let instances = vec![
            make_instance("a"),
            make_instance("b"),
            make_instance("c"),
            make_instance("d"),
        ];

        OrderingInstances::objects_in_definition_sequence(&instances);

        // Count instances in the ordering.
        let count = OrderingInstances::iter().count();
        assert_eq!(count, 4);
    }

    // -----------------------------------------------------------------------
    // OrderingInstances::first
    // -----------------------------------------------------------------------

    #[test]
    fn test_first_returns_first_instance_after_placing() {
        let instances = vec![make_instance("red"), make_instance("blue")];

        OrderingInstances::begin(&instances);
        OrderingInstances::place_next(0);
        OrderingInstances::place_next(1);

        assert_eq!(OrderingInstances::first(), Some(0));
    }

    #[test]
    fn test_first_returns_none_for_empty_list() {
        let instances: Vec<Instance> = Vec::new();

        OrderingInstances::begin(&instances);

        assert_eq!(OrderingInstances::first(), None);
    }

    // -----------------------------------------------------------------------
    // OrderingInstances::next
    // -----------------------------------------------------------------------

    #[test]
    fn test_next_returns_next_instance_in_ordering() {
        let instances = vec![make_instance("red"), make_instance("blue"), make_instance("green")];

        OrderingInstances::begin(&instances);
        OrderingInstances::place_next(0);
        OrderingInstances::place_next(1);
        OrderingInstances::place_next(2);

        assert_eq!(OrderingInstances::next(0), Some(1));
        assert_eq!(OrderingInstances::next(1), Some(2));
    }

    #[test]
    fn test_next_returns_none_for_last_instance() {
        let instances = vec![make_instance("red"), make_instance("blue")];

        OrderingInstances::begin(&instances);
        OrderingInstances::place_next(0);
        OrderingInstances::place_next(1);

        assert_eq!(OrderingInstances::next(1), None);
    }

    // -----------------------------------------------------------------------
    // OrderingInstances::iter
    // -----------------------------------------------------------------------

    #[test]
    fn test_iter_iterates_through_all_ordered_instances() {
        let instances = vec![
            make_instance("red"),
            make_instance("blue"),
            make_instance("green"),
        ];

        OrderingInstances::begin(&instances);
        OrderingInstances::place_next(0);
        OrderingInstances::place_next(1);
        OrderingInstances::place_next(2);

        let collected: Vec<usize> = OrderingInstances::iter().collect();
        assert_eq!(collected, vec![0, 1, 2]);
    }

    #[test]
    fn test_iter_returns_correct_sequence() {
        let instances = vec![
            make_instance("a"),
            make_instance("b"),
            make_instance("c"),
            make_instance("d"),
        ];

        OrderingInstances::begin(&instances);
        // Place in non-sequential order to verify iteration follows links.
        OrderingInstances::place_next(2);
        OrderingInstances::place_next(0);
        OrderingInstances::place_next(3);
        OrderingInstances::place_next(1);

        let collected: Vec<usize> = OrderingInstances::iter().collect();
        assert_eq!(collected, vec![2, 0, 3, 1]);
    }

    #[test]
    fn test_iter_works_with_empty_list() {
        let instances: Vec<Instance> = Vec::new();

        OrderingInstances::begin(&instances);

        let collected: Vec<usize> = OrderingInstances::iter().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_iter_returns_no_items_when_no_instances_placed() {
        let instances = vec![make_instance("red"), make_instance("blue")];

        OrderingInstances::begin(&instances);
        // Don't place any instances.

        let collected: Vec<usize> = OrderingInstances::iter().collect();
        assert!(collected.is_empty());
    }
}
