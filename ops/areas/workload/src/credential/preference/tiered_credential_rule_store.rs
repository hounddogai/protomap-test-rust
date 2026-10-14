use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered credential rules by code.
pub trait TieredCredentialRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredCredentialRule>;
    fn save(&self, item: &TieredCredentialRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredCredentialRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredCredentialRuleStore {
    pub items: RefCell<BTreeMap<String, TieredCredentialRule>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredCredentialRuleStore for MemoryTieredCredentialRuleStore {
    fn find(&self, code: &str) -> Option<TieredCredentialRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredCredentialRule) {
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

    fn list(&self) -> Vec<TieredCredentialRule> {
        let mut out: Vec<TieredCredentialRule> = Vec::new();
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

pub fn new_memory_tiered_credential_rule_store() -> Rc<dyn TieredCredentialRuleStore> {
    Rc::new(MemoryTieredCredentialRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredCredentialRuleStore {
    pub inner: Rc<dyn TieredCredentialRuleStore>,
    pub cache: RefCell<cache::TtlCache<TieredCredentialRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredCredentialRuleStore for CachedTieredCredentialRuleStore {
    fn find(&self, code: &str) -> Option<TieredCredentialRule> {
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

    fn save(&self, item: &TieredCredentialRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredCredentialRule> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_credential_rule_store(
    inner: Rc<dyn TieredCredentialRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredCredentialRuleStore> {
    Rc::new(CachedTieredCredentialRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredCredentialRule>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered credential rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialRuleQuery {
    pub title: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredCredentialRuleQuery {
    pub fn matches(&self, item: &TieredCredentialRule) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredCredentialRule]) -> Vec<TieredCredentialRule> {
        let matched = collections::filter(items, |candidate: &TieredCredentialRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_credential_rules(
    store: Rc<dyn TieredCredentialRuleStore>,
    query: &TieredCredentialRuleQuery,
) -> Vec<TieredCredentialRule> {
    query.apply(&store.list())
}
