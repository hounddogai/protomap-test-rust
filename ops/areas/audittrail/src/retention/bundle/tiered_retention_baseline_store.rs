use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered retention baselines by code.
pub trait TieredRetentionBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredRetentionBaseline>;
    fn save(&self, item: &TieredRetentionBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredRetentionBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredRetentionBaselineStore {
    pub items: RefCell<BTreeMap<String, TieredRetentionBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredRetentionBaselineStore for MemoryTieredRetentionBaselineStore {
    fn find(&self, code: &str) -> Option<TieredRetentionBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredRetentionBaseline) {
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

    fn list(&self) -> Vec<TieredRetentionBaseline> {
        let mut out: Vec<TieredRetentionBaseline> = Vec::new();
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

pub fn new_memory_tiered_retention_baseline_store() -> Rc<dyn TieredRetentionBaselineStore> {
    Rc::new(MemoryTieredRetentionBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredRetentionBaselineStore {
    pub inner: Rc<dyn TieredRetentionBaselineStore>,
    pub cache: RefCell<cache::TtlCache<TieredRetentionBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredRetentionBaselineStore for CachedTieredRetentionBaselineStore {
    fn find(&self, code: &str) -> Option<TieredRetentionBaseline> {
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

    fn save(&self, item: &TieredRetentionBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredRetentionBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_retention_baseline_store(
    inner: Rc<dyn TieredRetentionBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredRetentionBaselineStore> {
    Rc::new(CachedTieredRetentionBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredRetentionBaseline>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered retention baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionBaselineQuery {
    pub channel: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredRetentionBaselineQuery {
    pub fn matches(&self, item: &TieredRetentionBaseline) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredRetentionBaseline]) -> Vec<TieredRetentionBaseline> {
        let matched = collections::filter(items, |candidate: &TieredRetentionBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_retention_baselines(
    store: Rc<dyn TieredRetentionBaselineStore>,
    query: &TieredRetentionBaselineQuery,
) -> Vec<TieredRetentionBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredRetentionBaselineStore> = new_memory_tiered_retention_baseline_store();
        let item = new_tiered_retention_baseline("code-913", "label-393");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
