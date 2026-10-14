use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global release rules by code.
pub trait GlobalReleaseRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalReleaseRule>;
    fn save(&self, item: &GlobalReleaseRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalReleaseRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalReleaseRuleStore {
    pub items: RefCell<BTreeMap<String, GlobalReleaseRule>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalReleaseRuleStore for MemoryGlobalReleaseRuleStore {
    fn find(&self, code: &str) -> Option<GlobalReleaseRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalReleaseRule) {
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

    fn list(&self) -> Vec<GlobalReleaseRule> {
        let mut out: Vec<GlobalReleaseRule> = Vec::new();
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

pub fn new_memory_global_release_rule_store() -> Rc<dyn GlobalReleaseRuleStore> {
    Rc::new(MemoryGlobalReleaseRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalReleaseRuleStore {
    pub inner: Rc<dyn GlobalReleaseRuleStore>,
    pub cache: RefCell<cache::TtlCache<GlobalReleaseRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalReleaseRuleStore for CachedGlobalReleaseRuleStore {
    fn find(&self, code: &str) -> Option<GlobalReleaseRule> {
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

    fn save(&self, item: &GlobalReleaseRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalReleaseRule> {
        self.inner.list()
    }
}

pub fn new_cached_global_release_rule_store(
    inner: Rc<dyn GlobalReleaseRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalReleaseRuleStore> {
    Rc::new(CachedGlobalReleaseRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalReleaseRule>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global release rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseRuleQuery {
    pub source: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalReleaseRuleQuery {
    pub fn matches(&self, item: &GlobalReleaseRule) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalReleaseRule]) -> Vec<GlobalReleaseRule> {
        let matched = collections::filter(items, |candidate: &GlobalReleaseRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_release_rules(
    store: Rc<dyn GlobalReleaseRuleStore>,
    query: &GlobalReleaseRuleQuery,
) -> Vec<GlobalReleaseRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalReleaseRuleStore> = new_memory_global_release_rule_store();
        let item = new_global_release_rule("code-678", "name-537");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
