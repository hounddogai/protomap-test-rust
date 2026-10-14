use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rule records by code.
pub trait RuleRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RuleRecord>;
    fn save(&self, item: &RuleRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RuleRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRuleRecordStore {
    pub items: RefCell<BTreeMap<String, RuleRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl RuleRecordStore for MemoryRuleRecordStore {
    fn find(&self, code: &str) -> Option<RuleRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RuleRecord) {
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

    fn list(&self) -> Vec<RuleRecord> {
        let mut out: Vec<RuleRecord> = Vec::new();
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

pub fn new_memory_rule_record_store() -> Rc<dyn RuleRecordStore> {
    Rc::new(MemoryRuleRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRuleRecordStore {
    pub inner: Rc<dyn RuleRecordStore>,
    pub cache: RefCell<cache::TtlCache<RuleRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RuleRecordStore for CachedRuleRecordStore {
    fn find(&self, code: &str) -> Option<RuleRecord> {
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

    fn save(&self, item: &RuleRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RuleRecord> {
        self.inner.list()
    }
}

pub fn new_cached_rule_record_store(
    inner: Rc<dyn RuleRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RuleRecordStore> {
    Rc::new(CachedRuleRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RuleRecord>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rule records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleRecordQuery {
    pub title: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RuleRecordQuery {
    pub fn matches(&self, item: &RuleRecord) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RuleRecord]) -> Vec<RuleRecord> {
        let matched = collections::filter(items, |candidate: &RuleRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rule_records(store: Rc<dyn RuleRecordStore>, query: &RuleRecordQuery) -> Vec<RuleRecord> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RuleRecordStore> = new_memory_rule_record_store();
        let item = new_rule_record("code-833", "title-861");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
