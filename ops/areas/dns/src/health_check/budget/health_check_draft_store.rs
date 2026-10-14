use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps health check drafts by code.
pub trait HealthCheckDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<HealthCheckDraft>;
    fn save(&self, item: &HealthCheckDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<HealthCheckDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryHealthCheckDraftStore {
    pub items: RefCell<BTreeMap<String, HealthCheckDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl HealthCheckDraftStore for MemoryHealthCheckDraftStore {
    fn find(&self, code: &str) -> Option<HealthCheckDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &HealthCheckDraft) {
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

    fn list(&self) -> Vec<HealthCheckDraft> {
        let mut out: Vec<HealthCheckDraft> = Vec::new();
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

pub fn new_memory_health_check_draft_store() -> Rc<dyn HealthCheckDraftStore> {
    Rc::new(MemoryHealthCheckDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedHealthCheckDraftStore {
    pub inner: Rc<dyn HealthCheckDraftStore>,
    pub cache: RefCell<cache::TtlCache<HealthCheckDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl HealthCheckDraftStore for CachedHealthCheckDraftStore {
    fn find(&self, code: &str) -> Option<HealthCheckDraft> {
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

    fn save(&self, item: &HealthCheckDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<HealthCheckDraft> {
        self.inner.list()
    }
}

pub fn new_cached_health_check_draft_store(
    inner: Rc<dyn HealthCheckDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn HealthCheckDraftStore> {
    Rc::new(CachedHealthCheckDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<HealthCheckDraft>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects health check drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckDraftQuery {
    pub title: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl HealthCheckDraftQuery {
    pub fn matches(&self, item: &HealthCheckDraft) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[HealthCheckDraft]) -> Vec<HealthCheckDraft> {
        let matched = collections::filter(items, |candidate: &HealthCheckDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_health_check_drafts(
    store: Rc<dyn HealthCheckDraftStore>,
    query: &HealthCheckDraftQuery,
) -> Vec<HealthCheckDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn HealthCheckDraftStore> = new_memory_health_check_draft_store();
        let item = new_health_check_draft("code-407", "title-173");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
