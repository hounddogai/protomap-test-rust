use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy probe proposals by code.
pub trait LegacyProbeProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyProbeProposal>;
    fn save(&self, item: &LegacyProbeProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyProbeProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyProbeProposalStore {
    pub items: RefCell<BTreeMap<String, LegacyProbeProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyProbeProposalStore for MemoryLegacyProbeProposalStore {
    fn find(&self, code: &str) -> Option<LegacyProbeProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyProbeProposal) {
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

    fn list(&self) -> Vec<LegacyProbeProposal> {
        let mut out: Vec<LegacyProbeProposal> = Vec::new();
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

pub fn new_memory_legacy_probe_proposal_store() -> Rc<dyn LegacyProbeProposalStore> {
    Rc::new(MemoryLegacyProbeProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyProbeProposalStore {
    pub inner: Rc<dyn LegacyProbeProposalStore>,
    pub cache: RefCell<cache::TtlCache<LegacyProbeProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyProbeProposalStore for CachedLegacyProbeProposalStore {
    fn find(&self, code: &str) -> Option<LegacyProbeProposal> {
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

    fn save(&self, item: &LegacyProbeProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyProbeProposal> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_probe_proposal_store(
    inner: Rc<dyn LegacyProbeProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyProbeProposalStore> {
    Rc::new(CachedLegacyProbeProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyProbeProposal>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy probe proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeProposalQuery {
    pub source: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyProbeProposalQuery {
    pub fn matches(&self, item: &LegacyProbeProposal) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[LegacyProbeProposal]) -> Vec<LegacyProbeProposal> {
        let matched = collections::filter(items, |candidate: &LegacyProbeProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_probe_proposals(
    store: Rc<dyn LegacyProbeProposalStore>,
    query: &LegacyProbeProposalQuery,
) -> Vec<LegacyProbeProposal> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyProbeProposalStore> = new_memory_legacy_probe_proposal_store();
        let item = new_legacy_probe_proposal("code-589", "title-777");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
