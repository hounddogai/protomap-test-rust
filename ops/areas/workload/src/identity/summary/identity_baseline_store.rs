use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps identity baselines by code.
pub trait IdentityBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IdentityBaseline>;
    fn save(&self, item: &IdentityBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IdentityBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIdentityBaselineStore {
    pub items: RefCell<BTreeMap<String, IdentityBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl IdentityBaselineStore for MemoryIdentityBaselineStore {
    fn find(&self, code: &str) -> Option<IdentityBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IdentityBaseline) {
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

    fn list(&self) -> Vec<IdentityBaseline> {
        let mut out: Vec<IdentityBaseline> = Vec::new();
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

pub fn new_memory_identity_baseline_store() -> Rc<dyn IdentityBaselineStore> {
    Rc::new(MemoryIdentityBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIdentityBaselineStore {
    pub inner: Rc<dyn IdentityBaselineStore>,
    pub cache: RefCell<cache::TtlCache<IdentityBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IdentityBaselineStore for CachedIdentityBaselineStore {
    fn find(&self, code: &str) -> Option<IdentityBaseline> {
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

    fn save(&self, item: &IdentityBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IdentityBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_identity_baseline_store(
    inner: Rc<dyn IdentityBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IdentityBaselineStore> {
    Rc::new(CachedIdentityBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IdentityBaseline>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects identity baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBaselineQuery {
    pub segment: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IdentityBaselineQuery {
    pub fn matches(&self, item: &IdentityBaseline) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[IdentityBaseline]) -> Vec<IdentityBaseline> {
        let matched = collections::filter(items, |candidate: &IdentityBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_identity_baselines(
    store: Rc<dyn IdentityBaselineStore>,
    query: &IdentityBaselineQuery,
) -> Vec<IdentityBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn IdentityBaselineStore> = new_memory_identity_baseline_store();
        let item = new_identity_baseline("code-952", "label-742");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
