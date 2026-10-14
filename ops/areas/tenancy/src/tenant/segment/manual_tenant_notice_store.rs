use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual tenant notices by code.
pub trait ManualTenantNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualTenantNotice>;
    fn save(&self, item: &ManualTenantNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualTenantNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualTenantNoticeStore {
    pub items: RefCell<BTreeMap<String, ManualTenantNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualTenantNoticeStore for MemoryManualTenantNoticeStore {
    fn find(&self, code: &str) -> Option<ManualTenantNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualTenantNotice) {
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

    fn list(&self) -> Vec<ManualTenantNotice> {
        let mut out: Vec<ManualTenantNotice> = Vec::new();
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

pub fn new_memory_manual_tenant_notice_store() -> Rc<dyn ManualTenantNoticeStore> {
    Rc::new(MemoryManualTenantNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualTenantNoticeStore {
    pub inner: Rc<dyn ManualTenantNoticeStore>,
    pub cache: RefCell<cache::TtlCache<ManualTenantNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualTenantNoticeStore for CachedManualTenantNoticeStore {
    fn find(&self, code: &str) -> Option<ManualTenantNotice> {
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

    fn save(&self, item: &ManualTenantNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualTenantNotice> {
        self.inner.list()
    }
}

pub fn new_cached_manual_tenant_notice_store(
    inner: Rc<dyn ManualTenantNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualTenantNoticeStore> {
    Rc::new(CachedManualTenantNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualTenantNotice>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual tenant notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantNoticeQuery {
    pub name: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualTenantNoticeQuery {
    pub fn matches(&self, item: &ManualTenantNotice) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualTenantNotice]) -> Vec<ManualTenantNotice> {
        let matched = collections::filter(items, |candidate: &ManualTenantNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_tenant_notices(
    store: Rc<dyn ManualTenantNoticeStore>,
    query: &ManualTenantNoticeQuery,
) -> Vec<ManualTenantNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualTenantNoticeStore> = new_memory_manual_tenant_notice_store();
        let item = new_manual_tenant_notice("code-899", "name-170");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
