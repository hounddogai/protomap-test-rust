use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending postmortem overrides by code.
pub trait PendingPostmortemOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingPostmortemOverride>;
    fn save(&self, item: &PendingPostmortemOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingPostmortemOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingPostmortemOverrideStore {
    pub items: RefCell<BTreeMap<String, PendingPostmortemOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingPostmortemOverrideStore for MemoryPendingPostmortemOverrideStore {
    fn find(&self, code: &str) -> Option<PendingPostmortemOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingPostmortemOverride) {
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

    fn list(&self) -> Vec<PendingPostmortemOverride> {
        let mut out: Vec<PendingPostmortemOverride> = Vec::new();
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

pub fn new_memory_pending_postmortem_override_store() -> Rc<dyn PendingPostmortemOverrideStore> {
    Rc::new(MemoryPendingPostmortemOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingPostmortemOverrideStore {
    pub inner: Rc<dyn PendingPostmortemOverrideStore>,
    pub cache: RefCell<cache::TtlCache<PendingPostmortemOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingPostmortemOverrideStore for CachedPendingPostmortemOverrideStore {
    fn find(&self, code: &str) -> Option<PendingPostmortemOverride> {
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

    fn save(&self, item: &PendingPostmortemOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingPostmortemOverride> {
        self.inner.list()
    }
}

pub fn new_cached_pending_postmortem_override_store(
    inner: Rc<dyn PendingPostmortemOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingPostmortemOverrideStore> {
    Rc::new(CachedPendingPostmortemOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingPostmortemOverride>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending postmortem overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemOverrideQuery {
    pub unit: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingPostmortemOverrideQuery {
    pub fn matches(&self, item: &PendingPostmortemOverride) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
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

    pub fn apply(&self, items: &[PendingPostmortemOverride]) -> Vec<PendingPostmortemOverride> {
        let matched = collections::filter(items, |candidate: &PendingPostmortemOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_postmortem_overrides(
    store: Rc<dyn PendingPostmortemOverrideStore>,
    query: &PendingPostmortemOverrideQuery,
) -> Vec<PendingPostmortemOverride> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingPostmortemOverrideStore> = new_memory_pending_postmortem_override_store();
        let item = new_pending_postmortem_override("code-679", "title-667");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
