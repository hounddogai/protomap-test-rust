use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual health check baselines by code.
pub trait ManualHealthCheckBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualHealthCheckBaseline>;
    fn save(&self, item: &ManualHealthCheckBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualHealthCheckBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualHealthCheckBaselineStore {
    pub items: RefCell<BTreeMap<String, ManualHealthCheckBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualHealthCheckBaselineStore for MemoryManualHealthCheckBaselineStore {
    fn find(&self, code: &str) -> Option<ManualHealthCheckBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualHealthCheckBaseline) {
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

    fn list(&self) -> Vec<ManualHealthCheckBaseline> {
        let mut out: Vec<ManualHealthCheckBaseline> = Vec::new();
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

pub fn new_memory_manual_health_check_baseline_store() -> Rc<dyn ManualHealthCheckBaselineStore> {
    Rc::new(MemoryManualHealthCheckBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualHealthCheckBaselineStore {
    pub inner: Rc<dyn ManualHealthCheckBaselineStore>,
    pub cache: RefCell<cache::TtlCache<ManualHealthCheckBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualHealthCheckBaselineStore for CachedManualHealthCheckBaselineStore {
    fn find(&self, code: &str) -> Option<ManualHealthCheckBaseline> {
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

    fn save(&self, item: &ManualHealthCheckBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualHealthCheckBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_manual_health_check_baseline_store(
    inner: Rc<dyn ManualHealthCheckBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualHealthCheckBaselineStore> {
    Rc::new(CachedManualHealthCheckBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualHealthCheckBaseline>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual health check baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckBaselineQuery {
    pub name: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualHealthCheckBaselineQuery {
    pub fn matches(&self, item: &ManualHealthCheckBaseline) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualHealthCheckBaseline]) -> Vec<ManualHealthCheckBaseline> {
        let matched = collections::filter(items, |candidate: &ManualHealthCheckBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_health_check_baselines(
    store: Rc<dyn ManualHealthCheckBaselineStore>,
    query: &ManualHealthCheckBaselineQuery,
) -> Vec<ManualHealthCheckBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualHealthCheckBaselineStore> = new_memory_manual_health_check_baseline_store();
        let item = new_manual_health_check_baseline("code-370", "name-432");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
