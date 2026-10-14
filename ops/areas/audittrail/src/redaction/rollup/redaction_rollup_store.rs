use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps redaction rollups by code.
pub trait RedactionRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RedactionRollup>;
    fn save(&self, item: &RedactionRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RedactionRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRedactionRollupStore {
    pub items: RefCell<BTreeMap<String, RedactionRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl RedactionRollupStore for MemoryRedactionRollupStore {
    fn find(&self, code: &str) -> Option<RedactionRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RedactionRollup) {
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

    fn list(&self) -> Vec<RedactionRollup> {
        let mut out: Vec<RedactionRollup> = Vec::new();
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

pub fn new_memory_redaction_rollup_store() -> Rc<dyn RedactionRollupStore> {
    Rc::new(MemoryRedactionRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRedactionRollupStore {
    pub inner: Rc<dyn RedactionRollupStore>,
    pub cache: RefCell<cache::TtlCache<RedactionRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RedactionRollupStore for CachedRedactionRollupStore {
    fn find(&self, code: &str) -> Option<RedactionRollup> {
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

    fn save(&self, item: &RedactionRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RedactionRollup> {
        self.inner.list()
    }
}

pub fn new_cached_redaction_rollup_store(
    inner: Rc<dyn RedactionRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RedactionRollupStore> {
    Rc::new(CachedRedactionRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RedactionRollup>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects redaction rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionRollupQuery {
    pub name: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RedactionRollupQuery {
    pub fn matches(&self, item: &RedactionRollup) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[RedactionRollup]) -> Vec<RedactionRollup> {
        let matched = collections::filter(items, |candidate: &RedactionRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_redaction_rollups(
    store: Rc<dyn RedactionRollupStore>,
    query: &RedactionRollupQuery,
) -> Vec<RedactionRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RedactionRollupStore> = new_memory_redaction_rollup_store();
        let item = new_redaction_rollup("code-502", "name-676");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
