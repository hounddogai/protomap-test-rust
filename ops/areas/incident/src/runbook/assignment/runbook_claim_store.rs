use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps runbook claims by code.
pub trait RunbookClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RunbookClaim>;
    fn save(&self, item: &RunbookClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RunbookClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRunbookClaimStore {
    pub items: RefCell<BTreeMap<String, RunbookClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl RunbookClaimStore for MemoryRunbookClaimStore {
    fn find(&self, code: &str) -> Option<RunbookClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RunbookClaim) {
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

    fn list(&self) -> Vec<RunbookClaim> {
        let mut out: Vec<RunbookClaim> = Vec::new();
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

pub fn new_memory_runbook_claim_store() -> Rc<dyn RunbookClaimStore> {
    Rc::new(MemoryRunbookClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRunbookClaimStore {
    pub inner: Rc<dyn RunbookClaimStore>,
    pub cache: RefCell<cache::TtlCache<RunbookClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RunbookClaimStore for CachedRunbookClaimStore {
    fn find(&self, code: &str) -> Option<RunbookClaim> {
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

    fn save(&self, item: &RunbookClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RunbookClaim> {
        self.inner.list()
    }
}

pub fn new_cached_runbook_claim_store(
    inner: Rc<dyn RunbookClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RunbookClaimStore> {
    Rc::new(CachedRunbookClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RunbookClaim>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects runbook claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookClaimQuery {
    pub channel: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RunbookClaimQuery {
    pub fn matches(&self, item: &RunbookClaim) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RunbookClaim]) -> Vec<RunbookClaim> {
        let matched = collections::filter(items, |candidate: &RunbookClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_runbook_claims(store: Rc<dyn RunbookClaimStore>, query: &RunbookClaimQuery) -> Vec<RunbookClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RunbookClaimStore> = new_memory_runbook_claim_store();
        let item = new_runbook_claim("code-662", "label-42");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
