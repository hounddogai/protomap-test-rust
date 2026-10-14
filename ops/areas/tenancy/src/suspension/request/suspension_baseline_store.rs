use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps suspension baselines by code.
pub trait SuspensionBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SuspensionBaseline>;
    fn save(&self, item: &SuspensionBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SuspensionBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySuspensionBaselineStore {
    pub items: RefCell<BTreeMap<String, SuspensionBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl SuspensionBaselineStore for MemorySuspensionBaselineStore {
    fn find(&self, code: &str) -> Option<SuspensionBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SuspensionBaseline) {
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

    fn list(&self) -> Vec<SuspensionBaseline> {
        let mut out: Vec<SuspensionBaseline> = Vec::new();
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

pub fn new_memory_suspension_baseline_store() -> Rc<dyn SuspensionBaselineStore> {
    Rc::new(MemorySuspensionBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSuspensionBaselineStore {
    pub inner: Rc<dyn SuspensionBaselineStore>,
    pub cache: RefCell<cache::TtlCache<SuspensionBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SuspensionBaselineStore for CachedSuspensionBaselineStore {
    fn find(&self, code: &str) -> Option<SuspensionBaseline> {
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

    fn save(&self, item: &SuspensionBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SuspensionBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_suspension_baseline_store(
    inner: Rc<dyn SuspensionBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SuspensionBaselineStore> {
    Rc::new(CachedSuspensionBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SuspensionBaseline>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects suspension baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBaselineQuery {
    pub source: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SuspensionBaselineQuery {
    pub fn matches(&self, item: &SuspensionBaseline) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SuspensionBaseline]) -> Vec<SuspensionBaseline> {
        let matched = collections::filter(items, |candidate: &SuspensionBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_suspension_baselines(
    store: Rc<dyn SuspensionBaselineStore>,
    query: &SuspensionBaselineQuery,
) -> Vec<SuspensionBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SuspensionBaselineStore> = new_memory_suspension_baseline_store();
        let item = new_suspension_baseline("code-535", "name-750");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
