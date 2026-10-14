use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient freeze snapshots by code.
pub trait LenientFreezeSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientFreezeSnapshot>;
    fn save(&self, item: &LenientFreezeSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientFreezeSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientFreezeSnapshotStore {
    pub items: RefCell<BTreeMap<String, LenientFreezeSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientFreezeSnapshotStore for MemoryLenientFreezeSnapshotStore {
    fn find(&self, code: &str) -> Option<LenientFreezeSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientFreezeSnapshot) {
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

    fn list(&self) -> Vec<LenientFreezeSnapshot> {
        let mut out: Vec<LenientFreezeSnapshot> = Vec::new();
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

pub fn new_memory_lenient_freeze_snapshot_store() -> Rc<dyn LenientFreezeSnapshotStore> {
    Rc::new(MemoryLenientFreezeSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientFreezeSnapshotStore {
    pub inner: Rc<dyn LenientFreezeSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<LenientFreezeSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientFreezeSnapshotStore for CachedLenientFreezeSnapshotStore {
    fn find(&self, code: &str) -> Option<LenientFreezeSnapshot> {
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

    fn save(&self, item: &LenientFreezeSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientFreezeSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_freeze_snapshot_store(
    inner: Rc<dyn LenientFreezeSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientFreezeSnapshotStore> {
    Rc::new(CachedLenientFreezeSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientFreezeSnapshot>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient freeze snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeSnapshotQuery {
    pub country: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientFreezeSnapshotQuery {
    pub fn matches(&self, item: &LenientFreezeSnapshot) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientFreezeSnapshot]) -> Vec<LenientFreezeSnapshot> {
        let matched = collections::filter(items, |candidate: &LenientFreezeSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_freeze_snapshots(
    store: Rc<dyn LenientFreezeSnapshotStore>,
    query: &LenientFreezeSnapshotQuery,
) -> Vec<LenientFreezeSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientFreezeSnapshotStore> = new_memory_lenient_freeze_snapshot_store();
        let item = new_lenient_freeze_snapshot("code-28", "title-218");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
