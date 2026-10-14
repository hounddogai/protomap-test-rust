use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps peering entries by code.
pub trait PeeringEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PeeringEntry>;
    fn save(&self, item: &PeeringEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PeeringEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPeeringEntryStore {
    pub items: RefCell<BTreeMap<String, PeeringEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl PeeringEntryStore for MemoryPeeringEntryStore {
    fn find(&self, code: &str) -> Option<PeeringEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PeeringEntry) {
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

    fn list(&self) -> Vec<PeeringEntry> {
        let mut out: Vec<PeeringEntry> = Vec::new();
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

pub fn new_memory_peering_entry_store() -> Rc<dyn PeeringEntryStore> {
    Rc::new(MemoryPeeringEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPeeringEntryStore {
    pub inner: Rc<dyn PeeringEntryStore>,
    pub cache: RefCell<cache::TtlCache<PeeringEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PeeringEntryStore for CachedPeeringEntryStore {
    fn find(&self, code: &str) -> Option<PeeringEntry> {
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

    fn save(&self, item: &PeeringEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PeeringEntry> {
        self.inner.list()
    }
}

pub fn new_cached_peering_entry_store(
    inner: Rc<dyn PeeringEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PeeringEntryStore> {
    Rc::new(CachedPeeringEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PeeringEntry>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects peering entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringEntryQuery {
    pub channel: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PeeringEntryQuery {
    pub fn matches(&self, item: &PeeringEntry) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PeeringEntry]) -> Vec<PeeringEntry> {
        let matched = collections::filter(items, |candidate: &PeeringEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_peering_entries(store: Rc<dyn PeeringEntryStore>, query: &PeeringEntryQuery) -> Vec<PeeringEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PeeringEntryStore> = new_memory_peering_entry_store();
        let item = new_peering_entry("code-529", "title-368");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
