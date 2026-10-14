use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps evidence metrics by code.
pub trait EvidenceMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EvidenceMetric>;
    fn save(&self, item: &EvidenceMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EvidenceMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEvidenceMetricStore {
    pub items: RefCell<BTreeMap<String, EvidenceMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl EvidenceMetricStore for MemoryEvidenceMetricStore {
    fn find(&self, code: &str) -> Option<EvidenceMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EvidenceMetric) {
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

    fn list(&self) -> Vec<EvidenceMetric> {
        let mut out: Vec<EvidenceMetric> = Vec::new();
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

pub fn new_memory_evidence_metric_store() -> Rc<dyn EvidenceMetricStore> {
    Rc::new(MemoryEvidenceMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEvidenceMetricStore {
    pub inner: Rc<dyn EvidenceMetricStore>,
    pub cache: RefCell<cache::TtlCache<EvidenceMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EvidenceMetricStore for CachedEvidenceMetricStore {
    fn find(&self, code: &str) -> Option<EvidenceMetric> {
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

    fn save(&self, item: &EvidenceMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EvidenceMetric> {
        self.inner.list()
    }
}

pub fn new_cached_evidence_metric_store(
    inner: Rc<dyn EvidenceMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EvidenceMetricStore> {
    Rc::new(CachedEvidenceMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EvidenceMetric>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects evidence metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMetricQuery {
    pub title: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EvidenceMetricQuery {
    pub fn matches(&self, item: &EvidenceMetric) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EvidenceMetric]) -> Vec<EvidenceMetric> {
        let matched = collections::filter(items, |candidate: &EvidenceMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_evidence_metrics(store: Rc<dyn EvidenceMetricStore>, query: &EvidenceMetricQuery) -> Vec<EvidenceMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EvidenceMetricStore> = new_memory_evidence_metric_store();
        let item = new_evidence_metric("code-610", "title-38");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
