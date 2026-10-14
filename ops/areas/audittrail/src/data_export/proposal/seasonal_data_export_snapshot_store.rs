use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal data export snapshots by code.
pub trait SeasonalDataExportSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalDataExportSnapshot>;
    fn save(&self, item: &SeasonalDataExportSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalDataExportSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalDataExportSnapshotStore {
    pub items: RefCell<BTreeMap<String, SeasonalDataExportSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalDataExportSnapshotStore for MemorySeasonalDataExportSnapshotStore {
    fn find(&self, code: &str) -> Option<SeasonalDataExportSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalDataExportSnapshot) {
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

    fn list(&self) -> Vec<SeasonalDataExportSnapshot> {
        let mut out: Vec<SeasonalDataExportSnapshot> = Vec::new();
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

pub fn new_memory_seasonal_data_export_snapshot_store() -> Rc<dyn SeasonalDataExportSnapshotStore> {
    Rc::new(MemorySeasonalDataExportSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalDataExportSnapshotStore {
    pub inner: Rc<dyn SeasonalDataExportSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalDataExportSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalDataExportSnapshotStore for CachedSeasonalDataExportSnapshotStore {
    fn find(&self, code: &str) -> Option<SeasonalDataExportSnapshot> {
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

    fn save(&self, item: &SeasonalDataExportSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalDataExportSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_data_export_snapshot_store(
    inner: Rc<dyn SeasonalDataExportSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalDataExportSnapshotStore> {
    Rc::new(CachedSeasonalDataExportSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalDataExportSnapshot>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal data export snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportSnapshotQuery {
    pub scope: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalDataExportSnapshotQuery {
    pub fn matches(&self, item: &SeasonalDataExportSnapshot) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalDataExportSnapshot]) -> Vec<SeasonalDataExportSnapshot> {
        let matched = collections::filter(items, |candidate: &SeasonalDataExportSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_data_export_snapshots(
    store: Rc<dyn SeasonalDataExportSnapshotStore>,
    query: &SeasonalDataExportSnapshotQuery,
) -> Vec<SeasonalDataExportSnapshot> {
    query.apply(&store.list())
}
