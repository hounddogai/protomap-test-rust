use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient rule requests by code.
pub trait LenientRuleRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientRuleRequest>;
    fn save(&self, item: &LenientRuleRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientRuleRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientRuleRequestStore {
    pub items: RefCell<BTreeMap<String, LenientRuleRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientRuleRequestStore for MemoryLenientRuleRequestStore {
    fn find(&self, code: &str) -> Option<LenientRuleRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientRuleRequest) {
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

    fn list(&self) -> Vec<LenientRuleRequest> {
        let mut out: Vec<LenientRuleRequest> = Vec::new();
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

pub fn new_memory_lenient_rule_request_store() -> Rc<dyn LenientRuleRequestStore> {
    Rc::new(MemoryLenientRuleRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientRuleRequestStore {
    pub inner: Rc<dyn LenientRuleRequestStore>,
    pub cache: RefCell<cache::TtlCache<LenientRuleRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientRuleRequestStore for CachedLenientRuleRequestStore {
    fn find(&self, code: &str) -> Option<LenientRuleRequest> {
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

    fn save(&self, item: &LenientRuleRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientRuleRequest> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_rule_request_store(
    inner: Rc<dyn LenientRuleRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientRuleRequestStore> {
    Rc::new(CachedLenientRuleRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientRuleRequest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient rule requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleRequestQuery {
    pub name: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientRuleRequestQuery {
    pub fn matches(&self, item: &LenientRuleRequest) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientRuleRequest]) -> Vec<LenientRuleRequest> {
        let matched = collections::filter(items, |candidate: &LenientRuleRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_rule_requests(
    store: Rc<dyn LenientRuleRequestStore>,
    query: &LenientRuleRequestQuery,
) -> Vec<LenientRuleRequest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientRuleRequestStore> = new_memory_lenient_rule_request_store();
        let item = new_lenient_rule_request("code-246", "name-717");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
