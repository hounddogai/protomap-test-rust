use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps retention rules by code.
pub trait RetentionRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RetentionRule>;
    fn save(&self, item: &RetentionRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RetentionRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetentionRuleStore {
    pub items: RefCell<BTreeMap<String, RetentionRule>>,
    pub order: RefCell<Vec<String>>,
}

impl RetentionRuleStore for MemoryRetentionRuleStore {
    fn find(&self, code: &str) -> Option<RetentionRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RetentionRule) {
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

    fn list(&self) -> Vec<RetentionRule> {
        let mut out: Vec<RetentionRule> = Vec::new();
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

pub fn new_memory_retention_rule_store() -> Rc<dyn RetentionRuleStore> {
    Rc::new(MemoryRetentionRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRetentionRuleStore {
    pub inner: Rc<dyn RetentionRuleStore>,
    pub cache: RefCell<cache::TtlCache<RetentionRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RetentionRuleStore for CachedRetentionRuleStore {
    fn find(&self, code: &str) -> Option<RetentionRule> {
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

    fn save(&self, item: &RetentionRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RetentionRule> {
        self.inner.list()
    }
}

pub fn new_cached_retention_rule_store(
    inner: Rc<dyn RetentionRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RetentionRuleStore> {
    Rc::new(CachedRetentionRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RetentionRule>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects retention rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRuleQuery {
    pub name: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RetentionRuleQuery {
    pub fn matches(&self, item: &RetentionRule) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RetentionRule]) -> Vec<RetentionRule> {
        let matched = collections::filter(items, |candidate: &RetentionRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_retention_rules(store: Rc<dyn RetentionRuleStore>, query: &RetentionRuleQuery) -> Vec<RetentionRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RetentionRuleStore> = new_memory_retention_rule_store();
        let item = new_retention_rule("code-85", "name-848");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
