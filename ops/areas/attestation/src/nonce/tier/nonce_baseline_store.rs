use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps nonce baselines by code.
pub trait NonceBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<NonceBaseline>;
    fn save(&self, item: &NonceBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<NonceBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryNonceBaselineStore {
    pub items: RefCell<BTreeMap<String, NonceBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl NonceBaselineStore for MemoryNonceBaselineStore {
    fn find(&self, code: &str) -> Option<NonceBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &NonceBaseline) {
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

    fn list(&self) -> Vec<NonceBaseline> {
        let mut out: Vec<NonceBaseline> = Vec::new();
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

pub fn new_memory_nonce_baseline_store() -> Rc<dyn NonceBaselineStore> {
    Rc::new(MemoryNonceBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedNonceBaselineStore {
    pub inner: Rc<dyn NonceBaselineStore>,
    pub cache: RefCell<cache::TtlCache<NonceBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl NonceBaselineStore for CachedNonceBaselineStore {
    fn find(&self, code: &str) -> Option<NonceBaseline> {
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

    fn save(&self, item: &NonceBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<NonceBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_nonce_baseline_store(
    inner: Rc<dyn NonceBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn NonceBaselineStore> {
    Rc::new(CachedNonceBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<NonceBaseline>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects nonce baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceBaselineQuery {
    pub scope: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl NonceBaselineQuery {
    pub fn matches(&self, item: &NonceBaseline) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[NonceBaseline]) -> Vec<NonceBaseline> {
        let matched = collections::filter(items, |candidate: &NonceBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_nonce_baselines(store: Rc<dyn NonceBaselineStore>, query: &NonceBaselineQuery) -> Vec<NonceBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn NonceBaselineStore> = new_memory_nonce_baseline_store();
        let item = new_nonce_baseline("code-940", "title-647");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
