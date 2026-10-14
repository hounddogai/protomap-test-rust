use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps postmortem policies by code.
pub trait PostmortemPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PostmortemPolicy>;
    fn save(&self, item: &PostmortemPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PostmortemPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPostmortemPolicyStore {
    pub items: RefCell<BTreeMap<String, PostmortemPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl PostmortemPolicyStore for MemoryPostmortemPolicyStore {
    fn find(&self, code: &str) -> Option<PostmortemPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PostmortemPolicy) {
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

    fn list(&self) -> Vec<PostmortemPolicy> {
        let mut out: Vec<PostmortemPolicy> = Vec::new();
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

pub fn new_memory_postmortem_policy_store() -> Rc<dyn PostmortemPolicyStore> {
    Rc::new(MemoryPostmortemPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPostmortemPolicyStore {
    pub inner: Rc<dyn PostmortemPolicyStore>,
    pub cache: RefCell<cache::TtlCache<PostmortemPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PostmortemPolicyStore for CachedPostmortemPolicyStore {
    fn find(&self, code: &str) -> Option<PostmortemPolicy> {
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

    fn save(&self, item: &PostmortemPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PostmortemPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_postmortem_policy_store(
    inner: Rc<dyn PostmortemPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PostmortemPolicyStore> {
    Rc::new(CachedPostmortemPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PostmortemPolicy>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects postmortem policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPolicyQuery {
    pub source: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PostmortemPolicyQuery {
    pub fn matches(&self, item: &PostmortemPolicy) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PostmortemPolicy]) -> Vec<PostmortemPolicy> {
        let matched = collections::filter(items, |candidate: &PostmortemPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_postmortem_policies(
    store: Rc<dyn PostmortemPolicyStore>,
    query: &PostmortemPolicyQuery,
) -> Vec<PostmortemPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PostmortemPolicyStore> = new_memory_postmortem_policy_store();
        let item = new_postmortem_policy("code-870", "title-695");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
