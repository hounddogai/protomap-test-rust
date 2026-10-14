use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped forecast claims by code.
pub trait CappedForecastClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedForecastClaim>;
    fn save(&self, item: &CappedForecastClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedForecastClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedForecastClaimStore {
    pub items: RefCell<BTreeMap<String, CappedForecastClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedForecastClaimStore for MemoryCappedForecastClaimStore {
    fn find(&self, code: &str) -> Option<CappedForecastClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedForecastClaim) {
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

    fn list(&self) -> Vec<CappedForecastClaim> {
        let mut out: Vec<CappedForecastClaim> = Vec::new();
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

pub fn new_memory_capped_forecast_claim_store() -> Rc<dyn CappedForecastClaimStore> {
    Rc::new(MemoryCappedForecastClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedForecastClaimStore {
    pub inner: Rc<dyn CappedForecastClaimStore>,
    pub cache: RefCell<cache::TtlCache<CappedForecastClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedForecastClaimStore for CachedCappedForecastClaimStore {
    fn find(&self, code: &str) -> Option<CappedForecastClaim> {
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

    fn save(&self, item: &CappedForecastClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedForecastClaim> {
        self.inner.list()
    }
}

pub fn new_cached_capped_forecast_claim_store(
    inner: Rc<dyn CappedForecastClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedForecastClaimStore> {
    Rc::new(CachedCappedForecastClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedForecastClaim>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped forecast claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastClaimQuery {
    pub source: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedForecastClaimQuery {
    pub fn matches(&self, item: &CappedForecastClaim) -> bool {
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

    pub fn apply(&self, items: &[CappedForecastClaim]) -> Vec<CappedForecastClaim> {
        let matched = collections::filter(items, |candidate: &CappedForecastClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_forecast_claims(
    store: Rc<dyn CappedForecastClaimStore>,
    query: &CappedForecastClaimQuery,
) -> Vec<CappedForecastClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedForecastClaimStore> = new_memory_capped_forecast_claim_store();
        let item = new_capped_forecast_claim("code-994", "label-725");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
