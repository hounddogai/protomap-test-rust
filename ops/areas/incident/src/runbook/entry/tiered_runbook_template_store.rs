use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered runbook templates by code.
pub trait TieredRunbookTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredRunbookTemplate>;
    fn save(&self, item: &TieredRunbookTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredRunbookTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredRunbookTemplateStore {
    pub items: RefCell<BTreeMap<String, TieredRunbookTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredRunbookTemplateStore for MemoryTieredRunbookTemplateStore {
    fn find(&self, code: &str) -> Option<TieredRunbookTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredRunbookTemplate) {
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

    fn list(&self) -> Vec<TieredRunbookTemplate> {
        let mut out: Vec<TieredRunbookTemplate> = Vec::new();
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

pub fn new_memory_tiered_runbook_template_store() -> Rc<dyn TieredRunbookTemplateStore> {
    Rc::new(MemoryTieredRunbookTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredRunbookTemplateStore {
    pub inner: Rc<dyn TieredRunbookTemplateStore>,
    pub cache: RefCell<cache::TtlCache<TieredRunbookTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredRunbookTemplateStore for CachedTieredRunbookTemplateStore {
    fn find(&self, code: &str) -> Option<TieredRunbookTemplate> {
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

    fn save(&self, item: &TieredRunbookTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredRunbookTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_runbook_template_store(
    inner: Rc<dyn TieredRunbookTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredRunbookTemplateStore> {
    Rc::new(CachedTieredRunbookTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredRunbookTemplate>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered runbook templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookTemplateQuery {
    pub unit: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredRunbookTemplateQuery {
    pub fn matches(&self, item: &TieredRunbookTemplate) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredRunbookTemplate]) -> Vec<TieredRunbookTemplate> {
        let matched = collections::filter(items, |candidate: &TieredRunbookTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_runbook_templates(
    store: Rc<dyn TieredRunbookTemplateStore>,
    query: &TieredRunbookTemplateQuery,
) -> Vec<TieredRunbookTemplate> {
    query.apply(&store.list())
}
