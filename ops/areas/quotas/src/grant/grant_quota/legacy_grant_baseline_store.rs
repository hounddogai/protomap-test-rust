use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy grant baselines by code.
pub trait LegacyGrantBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyGrantBaseline>;
    fn save(&self, item: &LegacyGrantBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyGrantBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyGrantBaselineStore {
    pub items: RefCell<BTreeMap<String, LegacyGrantBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyGrantBaselineStore for MemoryLegacyGrantBaselineStore {
    fn find(&self, code: &str) -> Option<LegacyGrantBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyGrantBaseline) {
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

    fn list(&self) -> Vec<LegacyGrantBaseline> {
        let mut out: Vec<LegacyGrantBaseline> = Vec::new();
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

pub fn new_memory_legacy_grant_baseline_store() -> Rc<dyn LegacyGrantBaselineStore> {
    Rc::new(MemoryLegacyGrantBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyGrantBaselineStore {
    pub inner: Rc<dyn LegacyGrantBaselineStore>,
    pub cache: RefCell<cache::TtlCache<LegacyGrantBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyGrantBaselineStore for CachedLegacyGrantBaselineStore {
    fn find(&self, code: &str) -> Option<LegacyGrantBaseline> {
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

    fn save(&self, item: &LegacyGrantBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyGrantBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_grant_baseline_store(
    inner: Rc<dyn LegacyGrantBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyGrantBaselineStore> {
    Rc::new(CachedLegacyGrantBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyGrantBaseline>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy grant baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantBaselineQuery {
    pub name: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyGrantBaselineQuery {
    pub fn matches(&self, item: &LegacyGrantBaseline) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyGrantBaseline]) -> Vec<LegacyGrantBaseline> {
        let matched = collections::filter(items, |candidate: &LegacyGrantBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_grant_baselines(
    store: Rc<dyn LegacyGrantBaselineStore>,
    query: &LegacyGrantBaselineQuery,
) -> Vec<LegacyGrantBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyGrantBaselineStore> = new_memory_legacy_grant_baseline_store();
        let item = new_legacy_grant_baseline("code-496", "name-703");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
