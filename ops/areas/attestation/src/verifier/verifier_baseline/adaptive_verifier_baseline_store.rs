use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive verifier baselines by code.
pub trait AdaptiveVerifierBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveVerifierBaseline>;
    fn save(&self, item: &AdaptiveVerifierBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveVerifierBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveVerifierBaselineStore {
    pub items: RefCell<BTreeMap<String, AdaptiveVerifierBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveVerifierBaselineStore for MemoryAdaptiveVerifierBaselineStore {
    fn find(&self, code: &str) -> Option<AdaptiveVerifierBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveVerifierBaseline) {
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

    fn list(&self) -> Vec<AdaptiveVerifierBaseline> {
        let mut out: Vec<AdaptiveVerifierBaseline> = Vec::new();
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

pub fn new_memory_adaptive_verifier_baseline_store() -> Rc<dyn AdaptiveVerifierBaselineStore> {
    Rc::new(MemoryAdaptiveVerifierBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveVerifierBaselineStore {
    pub inner: Rc<dyn AdaptiveVerifierBaselineStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveVerifierBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveVerifierBaselineStore for CachedAdaptiveVerifierBaselineStore {
    fn find(&self, code: &str) -> Option<AdaptiveVerifierBaseline> {
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

    fn save(&self, item: &AdaptiveVerifierBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveVerifierBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_verifier_baseline_store(
    inner: Rc<dyn AdaptiveVerifierBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveVerifierBaselineStore> {
    Rc::new(CachedAdaptiveVerifierBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveVerifierBaseline>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive verifier baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveVerifierBaselineQuery {
    pub country: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveVerifierBaselineQuery {
    pub fn matches(&self, item: &AdaptiveVerifierBaseline) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveVerifierBaseline]) -> Vec<AdaptiveVerifierBaseline> {
        let matched = collections::filter(items, |candidate: &AdaptiveVerifierBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_verifier_baselines(
    store: Rc<dyn AdaptiveVerifierBaselineStore>,
    query: &AdaptiveVerifierBaselineQuery,
) -> Vec<AdaptiveVerifierBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveVerifierBaselineStore> = new_memory_adaptive_verifier_baseline_store();
        let item = new_adaptive_verifier_baseline("code-864", "title-688");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
