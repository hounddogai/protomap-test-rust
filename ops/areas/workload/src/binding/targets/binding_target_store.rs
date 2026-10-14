use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding targets by code.
pub trait BindingTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingTarget>;
    fn save(&self, item: &BindingTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingTargetStore {
    pub items: RefCell<BTreeMap<String, BindingTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingTargetStore for MemoryBindingTargetStore {
    fn find(&self, code: &str) -> Option<BindingTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingTarget) {
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

    fn list(&self) -> Vec<BindingTarget> {
        let mut out: Vec<BindingTarget> = Vec::new();
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

pub fn new_memory_binding_target_store() -> Rc<dyn BindingTargetStore> {
    Rc::new(MemoryBindingTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingTargetStore {
    pub inner: Rc<dyn BindingTargetStore>,
    pub cache: RefCell<cache::TtlCache<BindingTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingTargetStore for CachedBindingTargetStore {
    fn find(&self, code: &str) -> Option<BindingTarget> {
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

    fn save(&self, item: &BindingTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingTarget> {
        self.inner.list()
    }
}

pub fn new_cached_binding_target_store(
    inner: Rc<dyn BindingTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingTargetStore> {
    Rc::new(CachedBindingTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingTarget>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingTargetQuery {
    pub locale: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingTargetQuery {
    pub fn matches(&self, item: &BindingTarget) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BindingTarget]) -> Vec<BindingTarget> {
        let matched = collections::filter(items, |candidate: &BindingTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_targets(store: Rc<dyn BindingTargetStore>, query: &BindingTargetQuery) -> Vec<BindingTarget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BindingTargetStore> = new_memory_binding_target_store();
        let item = new_binding_target("code-318", "title-439");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
