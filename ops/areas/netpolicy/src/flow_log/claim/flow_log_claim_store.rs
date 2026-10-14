use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps flow log claims by code.
pub trait FlowLogClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FlowLogClaim>;
    fn save(&self, item: &FlowLogClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FlowLogClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFlowLogClaimStore {
    pub items: RefCell<BTreeMap<String, FlowLogClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl FlowLogClaimStore for MemoryFlowLogClaimStore {
    fn find(&self, code: &str) -> Option<FlowLogClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FlowLogClaim) {
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

    fn list(&self) -> Vec<FlowLogClaim> {
        let mut out: Vec<FlowLogClaim> = Vec::new();
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

pub fn new_memory_flow_log_claim_store() -> Rc<dyn FlowLogClaimStore> {
    Rc::new(MemoryFlowLogClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFlowLogClaimStore {
    pub inner: Rc<dyn FlowLogClaimStore>,
    pub cache: RefCell<cache::TtlCache<FlowLogClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FlowLogClaimStore for CachedFlowLogClaimStore {
    fn find(&self, code: &str) -> Option<FlowLogClaim> {
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

    fn save(&self, item: &FlowLogClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FlowLogClaim> {
        self.inner.list()
    }
}

pub fn new_cached_flow_log_claim_store(
    inner: Rc<dyn FlowLogClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FlowLogClaimStore> {
    Rc::new(CachedFlowLogClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FlowLogClaim>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects flow log claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogClaimQuery {
    pub country: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FlowLogClaimQuery {
    pub fn matches(&self, item: &FlowLogClaim) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FlowLogClaim]) -> Vec<FlowLogClaim> {
        let matched = collections::filter(items, |candidate: &FlowLogClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_flow_log_claims(store: Rc<dyn FlowLogClaimStore>, query: &FlowLogClaimQuery) -> Vec<FlowLogClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FlowLogClaimStore> = new_memory_flow_log_claim_store();
        let item = new_flow_log_claim("code-26", "label-582");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
