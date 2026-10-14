use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding summaries by code.
pub trait BindingSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingSummary>;
    fn save(&self, item: &BindingSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingSummaryStore {
    pub items: RefCell<BTreeMap<String, BindingSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingSummaryStore for MemoryBindingSummaryStore {
    fn find(&self, code: &str) -> Option<BindingSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingSummary) {
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

    fn list(&self) -> Vec<BindingSummary> {
        let mut out: Vec<BindingSummary> = Vec::new();
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

pub fn new_memory_binding_summary_store() -> Rc<dyn BindingSummaryStore> {
    Rc::new(MemoryBindingSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingSummaryStore {
    pub inner: Rc<dyn BindingSummaryStore>,
    pub cache: RefCell<cache::TtlCache<BindingSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingSummaryStore for CachedBindingSummaryStore {
    fn find(&self, code: &str) -> Option<BindingSummary> {
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

    fn save(&self, item: &BindingSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingSummary> {
        self.inner.list()
    }
}

pub fn new_cached_binding_summary_store(
    inner: Rc<dyn BindingSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingSummaryStore> {
    Rc::new(CachedBindingSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingSummary>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingSummaryQuery {
    pub country: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingSummaryQuery {
    pub fn matches(&self, item: &BindingSummary) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BindingSummary]) -> Vec<BindingSummary> {
        let matched = collections::filter(items, |candidate: &BindingSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_summaries(store: Rc<dyn BindingSummaryStore>, query: &BindingSummaryQuery) -> Vec<BindingSummary> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BindingSummaryStore> = new_memory_binding_summary_store();
        let item = new_binding_summary("code-857", "label-744");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
