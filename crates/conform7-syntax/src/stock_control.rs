//! Stock control types for the linguistic inventory system.
//!
//! The stock control system provides the inventory for all linguistic items
//! (verbs, verb forms, prepositions, nouns, etc.). Each item belongs to a
//! grammatical category and can be looked up by its allocation ID.
//!
//! # References
//!
//! - C reference: `services/linguistics-module/Chapter 1/Stock Control.w` —
//!   the `grammatical_category`, `linguistic_stock_item`, and
//!   `grammatical_usage` types.

use crate::linguistic_constants::Lcon;
use std::any::Any;

/// A grammatical category for classifying linguistic items.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
#[derive(Clone, Debug)]
pub struct GrammaticalCategory {
    /// The name of this category (e.g., "verb", "preposition").
    pub name: String,
    /// The number of items in this category.
    pub item_count: usize,
}

impl GrammaticalCategory {
    /// Create a new grammatical category with the given name.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
    pub fn new(name: &str) -> Self {
        GrammaticalCategory {
            name: name.to_string(),
            item_count: 0,
        }
    }
}

/// A linguistic stock item — an item in the linguistic inventory.
///
/// Each item belongs to a grammatical category and carries arbitrary data.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
pub struct LinguisticStockItem {
    /// The category this item belongs to.
    pub category: GrammaticalCategory,
    /// The data carried by this item.
    pub data: Box<dyn Any>,
}

impl LinguisticStockItem {
    /// Create a new linguistic stock item.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
    pub fn new(category: GrammaticalCategory, data: Box<dyn Any>) -> Self {
        LinguisticStockItem { category, data }
    }
}

impl std::fmt::Debug for LinguisticStockItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinguisticStockItem")
            .field("category", &self.category)
            .field("data", &"<opaque>")
            .finish()
    }
}

/// A reference to a linguistic stock item by its allocation ID.
pub type LinguisticStockItemRef = usize;

/// A reference to a grammatical usage by its index.
pub type GrammaticalUsageRef = usize;

/// A grammatical usage — a specific usage of a linguistic item.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
#[derive(Clone, Debug)]
pub struct GrammaticalUsage {
    /// The stock item being used.
    pub item: LinguisticStockItemRef,
    /// The language this usage applies to.
    pub language: String,
    /// Possible forms this usage can take, as Lcon references.
    pub possible_forms: Vec<Lcon>,
}

impl GrammaticalUsage {
    /// Create a new grammatical usage.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
    pub fn new(item: LinguisticStockItemRef, language: &str) -> Self {
        GrammaticalUsage {
            item,
            language: language.to_string(),
            possible_forms: Vec::new(),
        }
    }
}

/// The stock registry — manages categories, items, and usages.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
#[derive(Default)]
pub struct Stock {
    /// All registered categories.
    pub categories: Vec<GrammaticalCategory>,
    /// All registered stock items.
    pub items: Vec<LinguisticStockItem>,
    /// All registered grammatical usages.
    pub usages: Vec<GrammaticalUsage>,
}

impl Stock {
    /// Create a new empty stock registry.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
    pub fn new() -> Self {
        Stock {
            categories: Vec::new(),
            items: Vec::new(),
            usages: Vec::new(),
        }
    }

    /// Create a new grammatical category and return its index.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
    pub fn new_category(&mut self, name: &str) -> usize {
        let index = self.categories.len();
        self.categories.push(GrammaticalCategory::new(name));
        index
    }

    /// Create a new stock item, register it, and return its index.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
    pub fn new(&mut self, category_index: usize, data: Box<dyn Any>) -> LinguisticStockItemRef {
        let category = self.categories[category_index].clone();
        let index = self.items.len();
        self.items.push(LinguisticStockItem::new(category, data));
        // Increment the category's item count.
        self.categories[category_index].item_count += 1;
        index
    }

    /// Convert a stock item reference to an Lcon.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
    pub fn to_lcon(item: LinguisticStockItemRef) -> Lcon {
        Lcon::of_id(item)
    }

    /// Look up a stock item from an Lcon reference.
    ///
    /// Returns `None` if the Lcon has no ID or the ID is out of range.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
    pub fn from_lcon(&self, lcon: Lcon) -> Option<&LinguisticStockItem> {
        let id = lcon.get_id()?;
        self.items.get(id)
    }

    /// Create a new grammatical usage.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
    pub fn new_usage(&mut self, item: LinguisticStockItemRef, language: &str) -> GrammaticalUsageRef {
        let index = self.usages.len();
        self.usages.push(GrammaticalUsage::new(item, language));
        index
    }

    /// Add a possible form (as an Lcon) to a usage.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
    pub fn add_form_to_usage(&mut self, usage: GrammaticalUsageRef, form: Lcon) {
        if let Some(u) = self.usages.get_mut(usage) {
            u.possible_forms.push(form);
        }
    }

    /// Get the first form from a usage, if any.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
    pub fn first_form_in_usage(&self, usage: GrammaticalUsageRef) -> Option<Lcon> {
        self.usages.get(usage).and_then(|u| u.possible_forms.first().copied())
    }

    /// Check if a usage might be singular.
    ///
    /// Returns `true` if any possible form has singular number.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
    pub fn usage_might_be_singular(&self, usage: GrammaticalUsageRef) -> bool {
        self.usages
            .get(usage)
            .map(|u| {
                u.possible_forms
                    .iter()
                    .any(|f| f.get_number() == crate::linguistic_constants::SINGULAR_NUMBER)
            })
            .unwrap_or(false)
    }

    /// Check if a usage might be third person.
    ///
    /// Returns `true` if any possible form has third person.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
    pub fn usage_might_be_third_person(&self, usage: GrammaticalUsageRef) -> bool {
        self.usages
            .get(usage)
            .map(|u| {
                u.possible_forms
                    .iter()
                    .any(|f| f.get_person() == crate::linguistic_constants::THIRD_PERSON)
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linguistic_constants::*;

    #[test]
    fn test_new_category() {
        // Reference: services/linguistics-module/Chapter 1/Stock Control.w
        let mut stock = Stock::new();
        let cat = stock.new_category("verb");
        assert_eq!(stock.categories[cat].name, "verb");
        assert_eq!(stock.categories[cat].item_count, 0);
    }

    #[test]
    fn test_new_item() {
        // Reference: services/linguistics-module/Chapter 1/Stock Control.w
        let mut stock = Stock::new();
        let cat = stock.new_category("verb");
        let item = stock.new(cat, Box::new("test_verb".to_string()));
        assert_eq!(item, 0);
        assert_eq!(stock.categories[cat].item_count, 1);
    }

    #[test]
    fn test_to_lcon_and_from_lcon_round_trip() {
        // Reference: services/linguistics-module/Chapter 1/Stock Control.w
        let mut stock = Stock::new();
        let cat = stock.new_category("verb");
        let item = stock.new(cat, Box::new("test_verb".to_string()));

        let lcon = Stock::to_lcon(item);
        assert_eq!(lcon.get_id(), Some(0));

        let retrieved = stock.from_lcon(lcon);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_from_lcon_none_for_missing() {
        // Reference: services/linguistics-module/Chapter 1/Stock Control.w
        let stock = Stock::new();
        let lcon = Lcon::of_id(999);
        assert!(stock.from_lcon(lcon).is_none());
    }

    #[test]
    fn test_from_lcon_none_for_no_id() {
        // Reference: services/linguistics-module/Chapter 1/Stock Control.w
        let stock = Stock::new();
        let lcon = Lcon::new(); // no ID
        assert!(stock.from_lcon(lcon).is_none());
    }

    #[test]
    fn test_new_usage() {
        // Reference: services/linguistics-module/Chapter 1/Stock Control.w
        let mut stock = Stock::new();
        let cat = stock.new_category("verb");
        let item = stock.new(cat, Box::new("test".to_string()));
        let usage = stock.new_usage(item, "English");
        assert_eq!(stock.usages[usage].item, item);
        assert_eq!(stock.usages[usage].language, "English");
    }

    #[test]
    fn test_add_form_to_usage() {
        // Reference: services/linguistics-module/Chapter 1/Stock Control.w
        let mut stock = Stock::new();
        let cat = stock.new_category("verb");
        let item = stock.new(cat, Box::new("test".to_string()));
        let usage = stock.new_usage(item, "English");

        let form = Lcon::of_id(0).with_person(THIRD_PERSON).with_number(SINGULAR_NUMBER);
        stock.add_form_to_usage(usage, form);

        assert_eq!(stock.usages[usage].possible_forms.len(), 1);
        assert_eq!(stock.first_form_in_usage(usage), Some(form));
    }

    #[test]
    fn test_usage_might_be_singular() {
        // Reference: services/linguistics-module/Chapter 1/Stock Control.w
        let mut stock = Stock::new();
        let cat = stock.new_category("verb");
        let item = stock.new(cat, Box::new("test".to_string()));
        let usage = stock.new_usage(item, "English");

        let singular = Lcon::of_id(0).with_number(SINGULAR_NUMBER);
        let plural = Lcon::of_id(1).with_number(PLURAL_NUMBER);
        stock.add_form_to_usage(usage, singular);
        stock.add_form_to_usage(usage, plural);

        assert!(stock.usage_might_be_singular(usage));
    }

    #[test]
    fn test_usage_might_be_third_person() {
        // Reference: services/linguistics-module/Chapter 1/Stock Control.w
        let mut stock = Stock::new();
        let cat = stock.new_category("verb");
        let item = stock.new(cat, Box::new("test".to_string()));
        let usage = stock.new_usage(item, "English");

        let third = Lcon::of_id(0).with_person(THIRD_PERSON);
        let first = Lcon::of_id(1).with_person(FIRST_PERSON);
        stock.add_form_to_usage(usage, third);
        stock.add_form_to_usage(usage, first);

        assert!(stock.usage_might_be_third_person(usage));
    }

    #[test]
    fn test_usage_might_not_be_singular() {
        // Reference: services/linguistics-module/Chapter 1/Stock Control.w
        let mut stock = Stock::new();
        let cat = stock.new_category("verb");
        let item = stock.new(cat, Box::new("test".to_string()));
        let usage = stock.new_usage(item, "English");

        let plural = Lcon::of_id(0).with_number(PLURAL_NUMBER);
        stock.add_form_to_usage(usage, plural);

        assert!(!stock.usage_might_be_singular(usage));
    }
}
