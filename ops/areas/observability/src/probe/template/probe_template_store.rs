use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps probe templates by code.
pub trait ProbeTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ProbeTemplate>;
    fn save(&self, item: &ProbeTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ProbeTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryProbeTemplateStore {
    pub items: RefCell<BTreeMap<String, ProbeTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl ProbeTemplateStore for MemoryProbeTemplateStore {
    fn find(&self, code: &str) -> Option<ProbeTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ProbeTemplate) {
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

    fn list(&self) -> Vec<ProbeTemplate> {
        let mut out: Vec<ProbeTemplate> = Vec::new();
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

pub fn new_memory_probe_template_store() -> Rc<dyn ProbeTemplateStore> {
    Rc::new(MemoryProbeTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedProbeTemplateStore {
    pub inner: Rc<dyn ProbeTemplateStore>,
    pub cache: RefCell<cache::TtlCache<ProbeTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ProbeTemplateStore for CachedProbeTemplateStore {
    fn find(&self, code: &str) -> Option<ProbeTemplate> {
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

    fn save(&self, item: &ProbeTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ProbeTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_probe_template_store(
    inner: Rc<dyn ProbeTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ProbeTemplateStore> {
    Rc::new(CachedProbeTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ProbeTemplate>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects probe templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTemplateQuery {
    pub label: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ProbeTemplateQuery {
    pub fn matches(&self, item: &ProbeTemplate) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[ProbeTemplate]) -> Vec<ProbeTemplate> {
        let matched = collections::filter(items, |candidate: &ProbeTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_probe_templates(store: Rc<dyn ProbeTemplateStore>, query: &ProbeTemplateQuery) -> Vec<ProbeTemplate> {
    query.apply(&store.list())
}
