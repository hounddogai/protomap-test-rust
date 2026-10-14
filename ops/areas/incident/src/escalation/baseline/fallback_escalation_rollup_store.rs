use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback escalation rollups by code.
pub trait FallbackEscalationRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackEscalationRollup>;
    fn save(&self, item: &FallbackEscalationRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackEscalationRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackEscalationRollupStore {
    pub items: RefCell<BTreeMap<String, FallbackEscalationRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackEscalationRollupStore for MemoryFallbackEscalationRollupStore {
    fn find(&self, code: &str) -> Option<FallbackEscalationRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackEscalationRollup) {
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

    fn list(&self) -> Vec<FallbackEscalationRollup> {
        let mut out: Vec<FallbackEscalationRollup> = Vec::new();
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

pub fn new_memory_fallback_escalation_rollup_store() -> Rc<dyn FallbackEscalationRollupStore> {
    Rc::new(MemoryFallbackEscalationRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackEscalationRollupStore {
    pub inner: Rc<dyn FallbackEscalationRollupStore>,
    pub cache: RefCell<cache::TtlCache<FallbackEscalationRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackEscalationRollupStore for CachedFallbackEscalationRollupStore {
    fn find(&self, code: &str) -> Option<FallbackEscalationRollup> {
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

    fn save(&self, item: &FallbackEscalationRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackEscalationRollup> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_escalation_rollup_store(
    inner: Rc<dyn FallbackEscalationRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackEscalationRollupStore> {
    Rc::new(CachedFallbackEscalationRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackEscalationRollup>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback escalation rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEscalationRollupQuery {
    pub country: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackEscalationRollupQuery {
    pub fn matches(&self, item: &FallbackEscalationRollup) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackEscalationRollup]) -> Vec<FallbackEscalationRollup> {
        let matched = collections::filter(items, |candidate: &FallbackEscalationRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_escalation_rollups(
    store: Rc<dyn FallbackEscalationRollupStore>,
    query: &FallbackEscalationRollupQuery,
) -> Vec<FallbackEscalationRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackEscalationRollupStore> = new_memory_fallback_escalation_rollup_store();
        let item = new_fallback_escalation_rollup("code-986", "title-548");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
