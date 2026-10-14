use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending redaction slots by code.
pub trait PendingRedactionSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingRedactionSlot>;
    fn save(&self, item: &PendingRedactionSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingRedactionSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingRedactionSlotStore {
    pub items: RefCell<BTreeMap<String, PendingRedactionSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingRedactionSlotStore for MemoryPendingRedactionSlotStore {
    fn find(&self, code: &str) -> Option<PendingRedactionSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingRedactionSlot) {
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

    fn list(&self) -> Vec<PendingRedactionSlot> {
        let mut out: Vec<PendingRedactionSlot> = Vec::new();
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

pub fn new_memory_pending_redaction_slot_store() -> Rc<dyn PendingRedactionSlotStore> {
    Rc::new(MemoryPendingRedactionSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingRedactionSlotStore {
    pub inner: Rc<dyn PendingRedactionSlotStore>,
    pub cache: RefCell<cache::TtlCache<PendingRedactionSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingRedactionSlotStore for CachedPendingRedactionSlotStore {
    fn find(&self, code: &str) -> Option<PendingRedactionSlot> {
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

    fn save(&self, item: &PendingRedactionSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingRedactionSlot> {
        self.inner.list()
    }
}

pub fn new_cached_pending_redaction_slot_store(
    inner: Rc<dyn PendingRedactionSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingRedactionSlotStore> {
    Rc::new(CachedPendingRedactionSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingRedactionSlot>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending redaction slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionSlotQuery {
    pub scope: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingRedactionSlotQuery {
    pub fn matches(&self, item: &PendingRedactionSlot) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingRedactionSlot]) -> Vec<PendingRedactionSlot> {
        let matched = collections::filter(items, |candidate: &PendingRedactionSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_redaction_slots(
    store: Rc<dyn PendingRedactionSlotStore>,
    query: &PendingRedactionSlotQuery,
) -> Vec<PendingRedactionSlot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingRedactionSlotStore> = new_memory_pending_redaction_slot_store();
        let item = new_pending_redaction_slot("code-95", "label-706");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
