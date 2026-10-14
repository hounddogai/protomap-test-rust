use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional tenant notices by code.
pub trait RegionalTenantNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalTenantNotice>;
    fn save(&self, item: &RegionalTenantNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalTenantNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalTenantNoticeStore {
    pub items: RefCell<BTreeMap<String, RegionalTenantNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalTenantNoticeStore for MemoryRegionalTenantNoticeStore {
    fn find(&self, code: &str) -> Option<RegionalTenantNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalTenantNotice) {
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

    fn list(&self) -> Vec<RegionalTenantNotice> {
        let mut out: Vec<RegionalTenantNotice> = Vec::new();
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

pub fn new_memory_regional_tenant_notice_store() -> Rc<dyn RegionalTenantNoticeStore> {
    Rc::new(MemoryRegionalTenantNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalTenantNoticeStore {
    pub inner: Rc<dyn RegionalTenantNoticeStore>,
    pub cache: RefCell<cache::TtlCache<RegionalTenantNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalTenantNoticeStore for CachedRegionalTenantNoticeStore {
    fn find(&self, code: &str) -> Option<RegionalTenantNotice> {
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

    fn save(&self, item: &RegionalTenantNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalTenantNotice> {
        self.inner.list()
    }
}

pub fn new_cached_regional_tenant_notice_store(
    inner: Rc<dyn RegionalTenantNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalTenantNoticeStore> {
    Rc::new(CachedRegionalTenantNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalTenantNotice>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional tenant notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTenantNoticeQuery {
    pub namespace: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalTenantNoticeQuery {
    pub fn matches(&self, item: &RegionalTenantNotice) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalTenantNotice]) -> Vec<RegionalTenantNotice> {
        let matched = collections::filter(items, |candidate: &RegionalTenantNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_tenant_notices(
    store: Rc<dyn RegionalTenantNoticeStore>,
    query: &RegionalTenantNoticeQuery,
) -> Vec<RegionalTenantNotice> {
    query.apply(&store.list())
}
