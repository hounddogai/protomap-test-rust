use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps redaction targets by code.
pub trait RedactionTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RedactionTarget>;
    fn save(&self, item: &RedactionTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RedactionTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRedactionTargetStore {
    pub items: RefCell<BTreeMap<String, RedactionTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl RedactionTargetStore for MemoryRedactionTargetStore {
    fn find(&self, code: &str) -> Option<RedactionTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RedactionTarget) {
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

    fn list(&self) -> Vec<RedactionTarget> {
        let mut out: Vec<RedactionTarget> = Vec::new();
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

pub fn new_memory_redaction_target_store() -> Rc<dyn RedactionTargetStore> {
    Rc::new(MemoryRedactionTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRedactionTargetStore {
    pub inner: Rc<dyn RedactionTargetStore>,
    pub cache: RefCell<cache::TtlCache<RedactionTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RedactionTargetStore for CachedRedactionTargetStore {
    fn find(&self, code: &str) -> Option<RedactionTarget> {
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

    fn save(&self, item: &RedactionTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RedactionTarget> {
        self.inner.list()
    }
}

pub fn new_cached_redaction_target_store(
    inner: Rc<dyn RedactionTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RedactionTargetStore> {
    Rc::new(CachedRedactionTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RedactionTarget>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects redaction targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTargetQuery {
    pub source: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RedactionTargetQuery {
    pub fn matches(&self, item: &RedactionTarget) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RedactionTarget]) -> Vec<RedactionTarget> {
        let matched = collections::filter(items, |candidate: &RedactionTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_redaction_targets(
    store: Rc<dyn RedactionTargetStore>,
    query: &RedactionTargetQuery,
) -> Vec<RedactionTarget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RedactionTargetStore> = new_memory_redaction_target_store();
        let item = new_redaction_target("code-526", "title-244");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
