use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default release rules by code.
pub trait DefaultReleaseRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultReleaseRule>;
    fn save(&self, item: &DefaultReleaseRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultReleaseRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultReleaseRuleStore {
    pub items: RefCell<BTreeMap<String, DefaultReleaseRule>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultReleaseRuleStore for MemoryDefaultReleaseRuleStore {
    fn find(&self, code: &str) -> Option<DefaultReleaseRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultReleaseRule) {
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

    fn list(&self) -> Vec<DefaultReleaseRule> {
        let mut out: Vec<DefaultReleaseRule> = Vec::new();
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

pub fn new_memory_default_release_rule_store() -> Rc<dyn DefaultReleaseRuleStore> {
    Rc::new(MemoryDefaultReleaseRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultReleaseRuleStore {
    pub inner: Rc<dyn DefaultReleaseRuleStore>,
    pub cache: RefCell<cache::TtlCache<DefaultReleaseRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultReleaseRuleStore for CachedDefaultReleaseRuleStore {
    fn find(&self, code: &str) -> Option<DefaultReleaseRule> {
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

    fn save(&self, item: &DefaultReleaseRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultReleaseRule> {
        self.inner.list()
    }
}

pub fn new_cached_default_release_rule_store(
    inner: Rc<dyn DefaultReleaseRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultReleaseRuleStore> {
    Rc::new(CachedDefaultReleaseRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultReleaseRule>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default release rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseRuleQuery {
    pub scope: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultReleaseRuleQuery {
    pub fn matches(&self, item: &DefaultReleaseRule) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultReleaseRule]) -> Vec<DefaultReleaseRule> {
        let matched = collections::filter(items, |candidate: &DefaultReleaseRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_release_rules(
    store: Rc<dyn DefaultReleaseRuleStore>,
    query: &DefaultReleaseRuleQuery,
) -> Vec<DefaultReleaseRule> {
    query.apply(&store.list())
}
