use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps migration notices by code.
pub trait MigrationNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MigrationNotice>;
    fn save(&self, item: &MigrationNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MigrationNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMigrationNoticeStore {
    pub items: RefCell<BTreeMap<String, MigrationNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl MigrationNoticeStore for MemoryMigrationNoticeStore {
    fn find(&self, code: &str) -> Option<MigrationNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MigrationNotice) {
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

    fn list(&self) -> Vec<MigrationNotice> {
        let mut out: Vec<MigrationNotice> = Vec::new();
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

pub fn new_memory_migration_notice_store() -> Rc<dyn MigrationNoticeStore> {
    Rc::new(MemoryMigrationNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMigrationNoticeStore {
    pub inner: Rc<dyn MigrationNoticeStore>,
    pub cache: RefCell<cache::TtlCache<MigrationNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MigrationNoticeStore for CachedMigrationNoticeStore {
    fn find(&self, code: &str) -> Option<MigrationNotice> {
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

    fn save(&self, item: &MigrationNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MigrationNotice> {
        self.inner.list()
    }
}

pub fn new_cached_migration_notice_store(
    inner: Rc<dyn MigrationNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MigrationNoticeStore> {
    Rc::new(CachedMigrationNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MigrationNotice>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects migration notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationNoticeQuery {
    pub source: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MigrationNoticeQuery {
    pub fn matches(&self, item: &MigrationNotice) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MigrationNotice]) -> Vec<MigrationNotice> {
        let matched = collections::filter(items, |candidate: &MigrationNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_migration_notices(
    store: Rc<dyn MigrationNoticeStore>,
    query: &MigrationNoticeQuery,
) -> Vec<MigrationNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MigrationNoticeStore> = new_memory_migration_notice_store();
        let item = new_migration_notice("code-9", "title-199");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
