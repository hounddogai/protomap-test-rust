use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rule digests by code.
pub trait RuleDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RuleDigest>;
    fn save(&self, item: &RuleDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RuleDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRuleDigestStore {
    pub items: RefCell<BTreeMap<String, RuleDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl RuleDigestStore for MemoryRuleDigestStore {
    fn find(&self, code: &str) -> Option<RuleDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RuleDigest) {
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

    fn list(&self) -> Vec<RuleDigest> {
        let mut out: Vec<RuleDigest> = Vec::new();
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

pub fn new_memory_rule_digest_store() -> Rc<dyn RuleDigestStore> {
    Rc::new(MemoryRuleDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRuleDigestStore {
    pub inner: Rc<dyn RuleDigestStore>,
    pub cache: RefCell<cache::TtlCache<RuleDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RuleDigestStore for CachedRuleDigestStore {
    fn find(&self, code: &str) -> Option<RuleDigest> {
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

    fn save(&self, item: &RuleDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RuleDigest> {
        self.inner.list()
    }
}

pub fn new_cached_rule_digest_store(
    inner: Rc<dyn RuleDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RuleDigestStore> {
    Rc::new(CachedRuleDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RuleDigest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rule digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleDigestQuery {
    pub name: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RuleDigestQuery {
    pub fn matches(&self, item: &RuleDigest) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RuleDigest]) -> Vec<RuleDigest> {
        let matched = collections::filter(items, |candidate: &RuleDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rule_digests(store: Rc<dyn RuleDigestStore>, query: &RuleDigestQuery) -> Vec<RuleDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RuleDigestStore> = new_memory_rule_digest_store();
        let item = new_rule_digest("code-397", "name-528");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
