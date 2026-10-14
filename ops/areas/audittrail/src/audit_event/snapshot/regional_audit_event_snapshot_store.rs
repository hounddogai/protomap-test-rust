use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional audit event snapshots by code.
pub trait RegionalAuditEventSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalAuditEventSnapshot>;
    fn save(&self, item: &RegionalAuditEventSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalAuditEventSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalAuditEventSnapshotStore {
    pub items: RefCell<BTreeMap<String, RegionalAuditEventSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalAuditEventSnapshotStore for MemoryRegionalAuditEventSnapshotStore {
    fn find(&self, code: &str) -> Option<RegionalAuditEventSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalAuditEventSnapshot) {
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

    fn list(&self) -> Vec<RegionalAuditEventSnapshot> {
        let mut out: Vec<RegionalAuditEventSnapshot> = Vec::new();
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

pub fn new_memory_regional_audit_event_snapshot_store() -> Rc<dyn RegionalAuditEventSnapshotStore> {
    Rc::new(MemoryRegionalAuditEventSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalAuditEventSnapshotStore {
    pub inner: Rc<dyn RegionalAuditEventSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<RegionalAuditEventSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalAuditEventSnapshotStore for CachedRegionalAuditEventSnapshotStore {
    fn find(&self, code: &str) -> Option<RegionalAuditEventSnapshot> {
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

    fn save(&self, item: &RegionalAuditEventSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalAuditEventSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_regional_audit_event_snapshot_store(
    inner: Rc<dyn RegionalAuditEventSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalAuditEventSnapshotStore> {
    Rc::new(CachedRegionalAuditEventSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalAuditEventSnapshot>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional audit event snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventSnapshotQuery {
    pub title: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalAuditEventSnapshotQuery {
    pub fn matches(&self, item: &RegionalAuditEventSnapshot) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalAuditEventSnapshot]) -> Vec<RegionalAuditEventSnapshot> {
        let matched = collections::filter(items, |candidate: &RegionalAuditEventSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_audit_event_snapshots(
    store: Rc<dyn RegionalAuditEventSnapshotStore>,
    query: &RegionalAuditEventSnapshotQuery,
) -> Vec<RegionalAuditEventSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalAuditEventSnapshotStore> = new_memory_regional_audit_event_snapshot_store();
        let item = new_regional_audit_event_snapshot("code-776", "title-134");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
