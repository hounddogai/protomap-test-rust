use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement proposals by code.
pub trait MeasurementProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementProposal>;
    fn save(&self, item: &MeasurementProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementProposalStore {
    pub items: RefCell<BTreeMap<String, MeasurementProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementProposalStore for MemoryMeasurementProposalStore {
    fn find(&self, code: &str) -> Option<MeasurementProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementProposal) {
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

    fn list(&self) -> Vec<MeasurementProposal> {
        let mut out: Vec<MeasurementProposal> = Vec::new();
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

pub fn new_memory_measurement_proposal_store() -> Rc<dyn MeasurementProposalStore> {
    Rc::new(MemoryMeasurementProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementProposalStore {
    pub inner: Rc<dyn MeasurementProposalStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementProposalStore for CachedMeasurementProposalStore {
    fn find(&self, code: &str) -> Option<MeasurementProposal> {
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

    fn save(&self, item: &MeasurementProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementProposal> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_proposal_store(
    inner: Rc<dyn MeasurementProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementProposalStore> {
    Rc::new(CachedMeasurementProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementProposal>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementProposalQuery {
    pub region: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementProposalQuery {
    pub fn matches(&self, item: &MeasurementProposal) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MeasurementProposal]) -> Vec<MeasurementProposal> {
        let matched = collections::filter(items, |candidate: &MeasurementProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_proposals(
    store: Rc<dyn MeasurementProposalStore>,
    query: &MeasurementProposalQuery,
) -> Vec<MeasurementProposal> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MeasurementProposalStore> = new_memory_measurement_proposal_store();
        let item = new_measurement_proposal("code-641", "label-761");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
