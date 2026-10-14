use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps data export rules by code.
pub trait DataExportRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DataExportRule>;
    fn save(&self, item: &DataExportRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DataExportRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDataExportRuleStore {
    pub items: RefCell<BTreeMap<String, DataExportRule>>,
    pub order: RefCell<Vec<String>>,
}

impl DataExportRuleStore for MemoryDataExportRuleStore {
    fn find(&self, code: &str) -> Option<DataExportRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DataExportRule) {
        if !self.items.borrow().contains_key(&item.code) {
            self.order.borrow_mut().push(item.code.clone());
        }
        self.items.borrow_mut().insert(item.code.clone(), item.clone());
    }

    fn remove(&self, code: &str) -> bool {
        if !self.items.borrow().contains_key(code) {
            return false;
        }
        self.items.borrow_mut().remove(code);
        let mut kept: Vec<String> = Vec::new();
        for existing in self.order.borrow().iter() {
            if existing.as_str() != code {
                kept.push(existing.clone());
            }
        }
        *self.order.borrow_mut() = kept;
        true
    }

    fn list(&self) -> Vec<DataExportRule> {
        let mut out: Vec<DataExportRule> = Vec::new();
        for key in self.order.borrow().iter() {
            let found = self.items.borrow().get(key).cloned();
            let Some(found) = found else {
                continue;
            };
            out.push(found);
        }
        out
    }
}

pub fn new_memory_data_export_rule_store() -> Rc<dyn DataExportRuleStore> {
    Rc::new(MemoryDataExportRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDataExportRuleStore {
    pub inner: Rc<dyn DataExportRuleStore>,
    pub cache: RefCell<cache::TtlCache<DataExportRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DataExportRuleStore for CachedDataExportRuleStore {
    fn find(&self, code: &str) -> Option<DataExportRule> {
        let hit = self.cache.borrow_mut().get(code);
        let Some(hit) = hit else {
            self.misses.set(self.misses.get() + 1);
            let Some(found) = self.inner.find(code) else {
                return None;
            };
            self.cache.borrow_mut().put(code, &found);
            return Some(found);
        };
        self.hits.set(self.hits.get() + 1);
        Some(hit)
    }

    fn save(&self, item: &DataExportRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DataExportRule> {
        self.inner.list()
    }
}

pub fn new_cached_data_export_rule_store(
    inner: Rc<dyn DataExportRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DataExportRuleStore> {
    Rc::new(CachedDataExportRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DataExportRule>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects data export rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRuleQuery {
    pub name: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DataExportRuleQuery {
    pub fn matches(&self, item: &DataExportRule) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DataExportRule]) -> Vec<DataExportRule> {
        let matched = collections::filter(items, |candidate: &DataExportRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_data_export_rules(store: Rc<dyn DataExportRuleStore>, query: &DataExportRuleQuery) -> Vec<DataExportRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DataExportRuleStore> = new_memory_data_export_rule_store();
        let item = new_data_export_rule("code-476", "name-727");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
