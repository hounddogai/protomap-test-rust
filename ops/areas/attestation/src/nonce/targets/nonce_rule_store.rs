use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps nonce rules by code.
pub trait NonceRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<NonceRule>;
    fn save(&self, item: &NonceRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<NonceRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryNonceRuleStore {
    pub items: RefCell<BTreeMap<String, NonceRule>>,
    pub order: RefCell<Vec<String>>,
}

impl NonceRuleStore for MemoryNonceRuleStore {
    fn find(&self, code: &str) -> Option<NonceRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &NonceRule) {
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

    fn list(&self) -> Vec<NonceRule> {
        let mut out: Vec<NonceRule> = Vec::new();
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

pub fn new_memory_nonce_rule_store() -> Rc<dyn NonceRuleStore> {
    Rc::new(MemoryNonceRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedNonceRuleStore {
    pub inner: Rc<dyn NonceRuleStore>,
    pub cache: RefCell<cache::TtlCache<NonceRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl NonceRuleStore for CachedNonceRuleStore {
    fn find(&self, code: &str) -> Option<NonceRule> {
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

    fn save(&self, item: &NonceRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<NonceRule> {
        self.inner.list()
    }
}

pub fn new_cached_nonce_rule_store(
    inner: Rc<dyn NonceRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn NonceRuleStore> {
    Rc::new(CachedNonceRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<NonceRule>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects nonce rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceRuleQuery {
    pub channel: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl NonceRuleQuery {
    pub fn matches(&self, item: &NonceRule) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[NonceRule]) -> Vec<NonceRule> {
        let matched = collections::filter(items, |candidate: &NonceRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_nonce_rules(store: Rc<dyn NonceRuleStore>, query: &NonceRuleQuery) -> Vec<NonceRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn NonceRuleStore> = new_memory_nonce_rule_store();
        let item = new_nonce_rule("code-295", "title-107");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
