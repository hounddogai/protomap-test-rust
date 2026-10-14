use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global sink policies by code.
pub trait GlobalSinkPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalSinkPolicy>;
    fn save(&self, item: &GlobalSinkPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalSinkPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalSinkPolicyStore {
    pub items: RefCell<BTreeMap<String, GlobalSinkPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalSinkPolicyStore for MemoryGlobalSinkPolicyStore {
    fn find(&self, code: &str) -> Option<GlobalSinkPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalSinkPolicy) {
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

    fn list(&self) -> Vec<GlobalSinkPolicy> {
        let mut out: Vec<GlobalSinkPolicy> = Vec::new();
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

pub fn new_memory_global_sink_policy_store() -> Rc<dyn GlobalSinkPolicyStore> {
    Rc::new(MemoryGlobalSinkPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalSinkPolicyStore {
    pub inner: Rc<dyn GlobalSinkPolicyStore>,
    pub cache: RefCell<cache::TtlCache<GlobalSinkPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalSinkPolicyStore for CachedGlobalSinkPolicyStore {
    fn find(&self, code: &str) -> Option<GlobalSinkPolicy> {
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

    fn save(&self, item: &GlobalSinkPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalSinkPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_global_sink_policy_store(
    inner: Rc<dyn GlobalSinkPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalSinkPolicyStore> {
    Rc::new(CachedGlobalSinkPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalSinkPolicy>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global sink policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkPolicyQuery {
    pub namespace: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalSinkPolicyQuery {
    pub fn matches(&self, item: &GlobalSinkPolicy) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[GlobalSinkPolicy]) -> Vec<GlobalSinkPolicy> {
        let matched = collections::filter(items, |candidate: &GlobalSinkPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_sink_policies(
    store: Rc<dyn GlobalSinkPolicyStore>,
    query: &GlobalSinkPolicyQuery,
) -> Vec<GlobalSinkPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalSinkPolicyStore> = new_memory_global_sink_policy_store();
        let item = new_global_sink_policy("code-914", "title-284");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
