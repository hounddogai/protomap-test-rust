use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps evidence baselines by code.
pub trait EvidenceBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EvidenceBaseline>;
    fn save(&self, item: &EvidenceBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EvidenceBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEvidenceBaselineStore {
    pub items: RefCell<BTreeMap<String, EvidenceBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl EvidenceBaselineStore for MemoryEvidenceBaselineStore {
    fn find(&self, code: &str) -> Option<EvidenceBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EvidenceBaseline) {
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

    fn list(&self) -> Vec<EvidenceBaseline> {
        let mut out: Vec<EvidenceBaseline> = Vec::new();
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

pub fn new_memory_evidence_baseline_store() -> Rc<dyn EvidenceBaselineStore> {
    Rc::new(MemoryEvidenceBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEvidenceBaselineStore {
    pub inner: Rc<dyn EvidenceBaselineStore>,
    pub cache: RefCell<cache::TtlCache<EvidenceBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EvidenceBaselineStore for CachedEvidenceBaselineStore {
    fn find(&self, code: &str) -> Option<EvidenceBaseline> {
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

    fn save(&self, item: &EvidenceBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EvidenceBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_evidence_baseline_store(
    inner: Rc<dyn EvidenceBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EvidenceBaselineStore> {
    Rc::new(CachedEvidenceBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EvidenceBaseline>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects evidence baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBaselineQuery {
    pub region: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EvidenceBaselineQuery {
    pub fn matches(&self, item: &EvidenceBaseline) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EvidenceBaseline]) -> Vec<EvidenceBaseline> {
        let matched = collections::filter(items, |candidate: &EvidenceBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_evidence_baselines(
    store: Rc<dyn EvidenceBaselineStore>,
    query: &EvidenceBaselineQuery,
) -> Vec<EvidenceBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EvidenceBaselineStore> = new_memory_evidence_baseline_store();
        let item = new_evidence_baseline("code-337", "label-510");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
