use crate::knowledge::inference_subjects::InferenceSubject;

/// A permission for a subject to have a property.
///
/// Corresponds to `property_permission` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Property Permissions.w`, lines 24-35).
#[derive(Clone, Debug)]
pub struct PropertyPermission {
    /// The subject to whom permission is granted.
    pub property_owner: usize, // simplified: index into a subject registry
    /// The property that is permitted (simplified: a string name for now).
    pub property_granted: &'static str,
    /// Where this permission was granted (simplified: a string tag).
    pub where_granted: Option<&'static str>,
    /// Storage data for compilation (simplified: a string tag).
    pub storage_data: Option<&'static str>,
}

impl PropertyPermission {
    /// Find a permission for a subject/property pair.
    ///
    /// Searches the subject's permission list for a matching property.
    /// If `allow_inheritance` is true, follows the subject hierarchy upward.
    ///
    /// Corresponds to `PropertyPermissions::find` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Property Permissions.w`, lines 52-118).
    pub fn find(
        subject: &InferenceSubject,
        property: &str,
        subjects: &[InferenceSubject],
        permissions: &[PropertyPermission],
        allow_inheritance: bool,
    ) -> Option<usize> {
        // Search this subject's permissions
        for &pp_idx in &subject.permissions_list {
            if pp_idx < permissions.len() && permissions[pp_idx].property_granted == property {
                return Some(pp_idx);
            }
        }

        // If inheritance is allowed, walk up the hierarchy
        if allow_inheritance {
            let mut current = subject.broader_than;
            while let Some(idx) = current {
                if idx < subjects.len() {
                    let broader = &subjects[idx];
                    for &pp_idx in &broader.permissions_list {
                        if pp_idx < permissions.len()
                            && permissions[pp_idx].property_granted == property
                        {
                            return Some(pp_idx);
                        }
                    }
                    current = broader.broader_than;
                } else {
                    break;
                }
            }
        }

        None
    }

    /// Grant a new permission for a subject/property pair.
    ///
    /// Creates a new permission if one doesn't exist, or returns the existing one.
    ///
    /// Corresponds to `PropertyPermissions::grant` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Property Permissions.w`, lines 52-118).
    pub fn grant(
        subject: &mut InferenceSubject,
        property: &'static str,
        where_granted: Option<&'static str>,
        subject_idx: usize,
        subjects: &[InferenceSubject],
        permissions: &mut Vec<PropertyPermission>,
    ) -> usize {
        // Check if a permission already exists
        if let Some(idx) =
            PropertyPermission::find(subject, property, subjects, permissions, false)
        {
            return idx;
        }

        // Create a new permission
        let idx = permissions.len();
        permissions.push(PropertyPermission {
            property_owner: subject_idx,
            property_granted: property,
            where_granted,
            storage_data: None,
        });
        subject.permissions_list.push(idx);
        idx
    }

    /// Get the property name from this permission.
    ///
    /// Corresponds to `PropertyPermissions::get_property` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Property Permissions.w`, lines 131-145).
    pub fn get_property(&self) -> &'static str {
        self.property_granted
    }

    /// Get the owner subject index from this permission.
    ///
    /// Corresponds to `PropertyPermissions::get_subject` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Property Permissions.w`, lines 131-145).
    pub fn get_owner(&self) -> usize {
        self.property_owner
    }

    /// Get the storage data from this permission.
    ///
    /// Corresponds to `PropertyPermissions::get_storage_data` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Property Permissions.w`, lines 131-145).
    pub fn get_storage_data(&self) -> Option<&'static str> {
        self.storage_data
    }

    /// Get where this permission was granted.
    ///
    /// Corresponds to `PropertyPermissions::where_granted` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Property Permissions.w`, lines 131-145).
    pub fn where_granted(&self) -> Option<&'static str> {
        self.where_granted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::inference_subjects::InferenceSubject;

    #[test]
    fn test_grant_creates_new_permission() {
        let mut subject = InferenceSubject::new(0, None, None, None);
        let subjects = vec![subject.clone()];
        let mut permissions = Vec::new();
        let idx = PropertyPermission::grant(
            &mut subject,
            "colour",
            Some("test_sentence"),
            0,
            &subjects,
            &mut permissions,
        );

        assert_eq!(idx, 0);
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[idx].property_granted, "colour");
        assert_eq!(permissions[idx].where_granted, Some("test_sentence"));
        assert_eq!(subject.permissions_list.len(), 1);
        assert_eq!(subject.permissions_list[0], 0);
    }

    #[test]
    fn test_find_finds_existing_permission() {
        let mut subject = InferenceSubject::new(0, None, None, None);
        let subjects = vec![subject.clone()];
        let mut permissions = Vec::new();
        let idx = PropertyPermission::grant(
            &mut subject,
            "colour",
            None,
            0,
            &subjects,
            &mut permissions,
        );

        let found = PropertyPermission::find(
            &subject,
            "colour",
            &subjects,
            &permissions,
            false,
        );
        assert_eq!(found, Some(idx));
    }

    #[test]
    fn test_find_returns_none_when_no_permission() {
        let subject = InferenceSubject::new(0, None, None, None);
        let subjects = vec![];
        let permissions = vec![];

        let found = PropertyPermission::find(
            &subject,
            "colour",
            &subjects,
            &permissions,
            false,
        );
        assert_eq!(found, None);
    }

    #[test]
    fn test_grant_returns_existing_permission() {
        let mut subject = InferenceSubject::new(0, None, None, None);
        let subjects = vec![subject.clone()];
        let mut permissions = Vec::new();
        let idx1 = PropertyPermission::grant(
            &mut subject,
            "colour",
            Some("first"),
            0,
            &subjects,
            &mut permissions,
        );
        let idx2 = PropertyPermission::grant(
            &mut subject,
            "colour",
            Some("second"),
            0,
            &subjects,
            &mut permissions,
        );

        // Should return the same index (existing permission)
        assert_eq!(idx1, idx2);
        assert_eq!(permissions.len(), 1);
    }

    #[test]
    fn test_find_follows_hierarchy() {
        // Create a hierarchy: root -> parent -> child
        let root = InferenceSubject::new(0, None, None, Some("root"));
        let mut parent = InferenceSubject::new(0, Some(0), None, Some("parent"));
        let child = InferenceSubject::new(0, Some(1), None, Some("child"));

        let subjects = vec![root.clone(), parent.clone(), child.clone()];
        let mut permissions = Vec::new();

        let pp_idx = PropertyPermission::grant(
            &mut parent,
            "colour",
            None,
            1, // parent is at index 1 in subjects
            &subjects,
            &mut permissions,
        );

        // Update the subjects array
        let subjects = vec![root, parent, child];

        // Find on child with inheritance should find parent's permission
        let found = PropertyPermission::find(
            &subjects[2], // child
            "colour",
            &subjects,
            &permissions,
            true, // allow inheritance
        );
        assert_eq!(found, Some(pp_idx));
    }

    #[test]
    fn test_find_does_not_follow_hierarchy_when_disabled() {
        let mut root = InferenceSubject::new(0, None, None, Some("root"));
        let child = InferenceSubject::new(0, Some(0), None, Some("child"));

        let subjects = vec![root.clone(), child.clone()];
        let mut permissions = Vec::new();

        PropertyPermission::grant(&mut root, "colour", None, 0, &subjects, &mut permissions);

        let subjects = vec![root, child];

        // Find on child without inheritance should NOT find root's permission
        let found = PropertyPermission::find(
            &subjects[1], // child
            "colour",
            &subjects,
            &permissions,
            false, // no inheritance
        );
        assert_eq!(found, None);
    }

    #[test]
    fn test_get_property_returns_correct_name() {
        let pp = PropertyPermission {
            property_owner: 0,
            property_granted: "colour",
            where_granted: None,
            storage_data: None,
        };
        assert_eq!(pp.get_property(), "colour");
    }

    #[test]
    fn test_get_owner_returns_correct_index() {
        let pp = PropertyPermission {
            property_owner: 42,
            property_granted: "colour",
            where_granted: None,
            storage_data: None,
        };
        assert_eq!(pp.get_owner(), 42);
    }

    #[test]
    fn test_get_storage_data_returns_none_by_default() {
        let pp = PropertyPermission {
            property_owner: 0,
            property_granted: "colour",
            where_granted: None,
            storage_data: None,
        };
        assert_eq!(pp.get_storage_data(), None);
    }

    #[test]
    fn test_get_storage_data_returns_set_value() {
        let pp = PropertyPermission {
            property_owner: 0,
            property_granted: "colour",
            where_granted: None,
            storage_data: Some("compiled_data"),
        };
        assert_eq!(pp.get_storage_data(), Some("compiled_data"));
    }

    #[test]
    fn test_where_granted_returns_none_by_default() {
        let pp = PropertyPermission {
            property_owner: 0,
            property_granted: "colour",
            where_granted: None,
            storage_data: None,
        };
        assert_eq!(pp.where_granted(), None);
    }

    #[test]
    fn test_where_granted_returns_set_value() {
        let pp = PropertyPermission {
            property_owner: 0,
            property_granted: "colour",
            where_granted: Some("sentence_42"),
            storage_data: None,
        };
        assert_eq!(pp.where_granted(), Some("sentence_42"));
    }

    #[test]
    fn test_grant_multiple_different_properties() {
        let mut subject = InferenceSubject::new(0, None, None, None);
        let subjects = vec![subject.clone()];
        let mut permissions = Vec::new();
        let idx1 = PropertyPermission::grant(
            &mut subject,
            "colour",
            None,
            0,
            &subjects,
            &mut permissions,
        );
        let idx2 = PropertyPermission::grant(
            &mut subject,
            "capacity",
            None,
            0,
            &subjects,
            &mut permissions,
        );

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(permissions.len(), 2);
        assert_eq!(subject.permissions_list.len(), 2);
    }

    #[test]
    fn test_grant_sets_property_owner() {
        let mut subject = InferenceSubject::new(0, None, None, None);
        let subjects = vec![subject.clone()];
        let mut permissions = Vec::new();

        let idx = PropertyPermission::grant(
            &mut subject,
            "colour",
            None,
            0,
            &subjects,
            &mut permissions,
        );

        assert_eq!(permissions[idx].property_owner, 0);
    }

    #[test]
    fn test_grant_sets_correct_owner_index() {
        let subject1 = InferenceSubject::new(0, None, None, Some("subject1"));
        let mut subject2 = InferenceSubject::new(0, None, None, Some("subject2"));
        let subjects = vec![subject1.clone(), subject2.clone()];
        let mut permissions = Vec::new();

        // Grant on subject2 (index 1)
        let idx = PropertyPermission::grant(
            &mut subject2,
            "colour",
            None,
            1,
            &subjects,
            &mut permissions,
        );

        assert_eq!(permissions[idx].property_owner, 1);
    }
}

