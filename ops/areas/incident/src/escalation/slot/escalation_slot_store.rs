use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps escalation slots by code.
pub trait EscalationSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EscalationSlot>;
    fn save(&self, item: &EscalationSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EscalationSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEscalationSlotStore {
    pub items: RefCell<BTreeMap<String, EscalationSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl EscalationSlotStore for MemoryEscalationSlotStore {
    fn find(&self, code: &str) -> Option<EscalationSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EscalationSlot) {
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

    fn list(&self) -> Vec<EscalationSlot> {
        let mut out: Vec<EscalationSlot> = Vec::new();
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

pub fn new_memory_escalation_slot_store() -> Rc<dyn EscalationSlotStore> {
    Rc::new(MemoryEscalationSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEscalationSlotStore {
    pub inner: Rc<dyn EscalationSlotStore>,
    pub cache: RefCell<cache::TtlCache<EscalationSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EscalationSlotStore for CachedEscalationSlotStore {
    fn find(&self, code: &str) -> Option<EscalationSlot> {
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

    fn save(&self, item: &EscalationSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EscalationSlot> {
        self.inner.list()
    }
}

pub fn new_cached_escalation_slot_store(
    inner: Rc<dyn EscalationSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EscalationSlotStore> {
    Rc::new(CachedEscalationSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EscalationSlot>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects escalation slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSlotQuery {
    pub label: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EscalationSlotQuery {
    pub fn matches(&self, item: &EscalationSlot) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EscalationSlot]) -> Vec<EscalationSlot> {
        let matched = collections::filter(items, |candidate: &EscalationSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_escalation_slots(store: Rc<dyn EscalationSlotStore>, query: &EscalationSlotQuery) -> Vec<EscalationSlot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EscalationSlotStore> = new_memory_escalation_slot_store();
        let item = new_escalation_slot("code-113", "label-607");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
