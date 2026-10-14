use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps credential rollups by code.
pub trait CredentialRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CredentialRollup>;
    fn save(&self, item: &CredentialRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CredentialRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCredentialRollupStore {
    pub items: RefCell<BTreeMap<String, CredentialRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl CredentialRollupStore for MemoryCredentialRollupStore {
    fn find(&self, code: &str) -> Option<CredentialRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CredentialRollup) {
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

    fn list(&self) -> Vec<CredentialRollup> {
        let mut out: Vec<CredentialRollup> = Vec::new();
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

pub fn new_memory_credential_rollup_store() -> Rc<dyn CredentialRollupStore> {
    Rc::new(MemoryCredentialRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCredentialRollupStore {
    pub inner: Rc<dyn CredentialRollupStore>,
    pub cache: RefCell<cache::TtlCache<CredentialRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CredentialRollupStore for CachedCredentialRollupStore {
    fn find(&self, code: &str) -> Option<CredentialRollup> {
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

    fn save(&self, item: &CredentialRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CredentialRollup> {
        self.inner.list()
    }
}

pub fn new_cached_credential_rollup_store(
    inner: Rc<dyn CredentialRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CredentialRollupStore> {
    Rc::new(CachedCredentialRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CredentialRollup>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects credential rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRollupQuery {
    pub channel: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CredentialRollupQuery {
    pub fn matches(&self, item: &CredentialRollup) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CredentialRollup]) -> Vec<CredentialRollup> {
        let matched = collections::filter(items, |candidate: &CredentialRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_credential_rollups(
    store: Rc<dyn CredentialRollupStore>,
    query: &CredentialRollupQuery,
) -> Vec<CredentialRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CredentialRollupStore> = new_memory_credential_rollup_store();
        let item = new_credential_rollup("code-816", "label-713");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
