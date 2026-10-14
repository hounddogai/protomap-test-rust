use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps audit event baselines by code.
pub trait AuditEventBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AuditEventBaseline>;
    fn save(&self, item: &AuditEventBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AuditEventBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAuditEventBaselineStore {
    pub items: RefCell<BTreeMap<String, AuditEventBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl AuditEventBaselineStore for MemoryAuditEventBaselineStore {
    fn find(&self, code: &str) -> Option<AuditEventBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AuditEventBaseline) {
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

    fn list(&self) -> Vec<AuditEventBaseline> {
        let mut out: Vec<AuditEventBaseline> = Vec::new();
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

pub fn new_memory_audit_event_baseline_store() -> Rc<dyn AuditEventBaselineStore> {
    Rc::new(MemoryAuditEventBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAuditEventBaselineStore {
    pub inner: Rc<dyn AuditEventBaselineStore>,
    pub cache: RefCell<cache::TtlCache<AuditEventBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AuditEventBaselineStore for CachedAuditEventBaselineStore {
    fn find(&self, code: &str) -> Option<AuditEventBaseline> {
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

    fn save(&self, item: &AuditEventBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AuditEventBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_audit_event_baseline_store(
    inner: Rc<dyn AuditEventBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AuditEventBaselineStore> {
    Rc::new(CachedAuditEventBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AuditEventBaseline>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects audit event baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBaselineQuery {
    pub title: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AuditEventBaselineQuery {
    pub fn matches(&self, item: &AuditEventBaseline) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AuditEventBaseline]) -> Vec<AuditEventBaseline> {
        let matched = collections::filter(items, |candidate: &AuditEventBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_audit_event_baselines(
    store: Rc<dyn AuditEventBaselineStore>,
    query: &AuditEventBaselineQuery,
) -> Vec<AuditEventBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AuditEventBaselineStore> = new_memory_audit_event_baseline_store();
        let item = new_audit_event_baseline("code-231", "title-257");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
