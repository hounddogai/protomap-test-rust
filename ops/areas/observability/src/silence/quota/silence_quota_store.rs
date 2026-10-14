use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps silence quotas by code.
pub trait SilenceQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SilenceQuota>;
    fn save(&self, item: &SilenceQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SilenceQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySilenceQuotaStore {
    pub items: RefCell<BTreeMap<String, SilenceQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl SilenceQuotaStore for MemorySilenceQuotaStore {
    fn find(&self, code: &str) -> Option<SilenceQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SilenceQuota) {
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

    fn list(&self) -> Vec<SilenceQuota> {
        let mut out: Vec<SilenceQuota> = Vec::new();
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

pub fn new_memory_silence_quota_store() -> Rc<dyn SilenceQuotaStore> {
    Rc::new(MemorySilenceQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSilenceQuotaStore {
    pub inner: Rc<dyn SilenceQuotaStore>,
    pub cache: RefCell<cache::TtlCache<SilenceQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SilenceQuotaStore for CachedSilenceQuotaStore {
    fn find(&self, code: &str) -> Option<SilenceQuota> {
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

    fn save(&self, item: &SilenceQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SilenceQuota> {
        self.inner.list()
    }
}

pub fn new_cached_silence_quota_store(
    inner: Rc<dyn SilenceQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SilenceQuotaStore> {
    Rc::new(CachedSilenceQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SilenceQuota>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects silence quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceQuotaQuery {
    pub unit: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SilenceQuotaQuery {
    pub fn matches(&self, item: &SilenceQuota) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SilenceQuota]) -> Vec<SilenceQuota> {
        let matched = collections::filter(items, |candidate: &SilenceQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_silence_quotas(store: Rc<dyn SilenceQuotaStore>, query: &SilenceQuotaQuery) -> Vec<SilenceQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SilenceQuotaStore> = new_memory_silence_quota_store();
        let item = new_silence_quota("code-255", "label-531");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
