use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary timeline rules by code.
pub trait PrimaryTimelineRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryTimelineRule>;
    fn save(&self, item: &PrimaryTimelineRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryTimelineRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryTimelineRuleStore {
    pub items: RefCell<BTreeMap<String, PrimaryTimelineRule>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryTimelineRuleStore for MemoryPrimaryTimelineRuleStore {
    fn find(&self, code: &str) -> Option<PrimaryTimelineRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryTimelineRule) {
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

    fn list(&self) -> Vec<PrimaryTimelineRule> {
        let mut out: Vec<PrimaryTimelineRule> = Vec::new();
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

pub fn new_memory_primary_timeline_rule_store() -> Rc<dyn PrimaryTimelineRuleStore> {
    Rc::new(MemoryPrimaryTimelineRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryTimelineRuleStore {
    pub inner: Rc<dyn PrimaryTimelineRuleStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryTimelineRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryTimelineRuleStore for CachedPrimaryTimelineRuleStore {
    fn find(&self, code: &str) -> Option<PrimaryTimelineRule> {
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

    fn save(&self, item: &PrimaryTimelineRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryTimelineRule> {
        self.inner.list()
    }
}

pub fn new_cached_primary_timeline_rule_store(
    inner: Rc<dyn PrimaryTimelineRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryTimelineRuleStore> {
    Rc::new(CachedPrimaryTimelineRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryTimelineRule>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary timeline rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTimelineRuleQuery {
    pub namespace: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryTimelineRuleQuery {
    pub fn matches(&self, item: &PrimaryTimelineRule) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryTimelineRule]) -> Vec<PrimaryTimelineRule> {
        let matched = collections::filter(items, |candidate: &PrimaryTimelineRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_timeline_rules(
    store: Rc<dyn PrimaryTimelineRuleStore>,
    query: &PrimaryTimelineRuleQuery,
) -> Vec<PrimaryTimelineRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryTimelineRuleStore> = new_memory_primary_timeline_rule_store();
        let item = new_primary_timeline_rule("code-441", "title-947");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
