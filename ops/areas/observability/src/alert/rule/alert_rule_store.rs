use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps alert rules by code.
pub trait AlertRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AlertRule>;
    fn save(&self, item: &AlertRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AlertRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAlertRuleStore {
    pub items: RefCell<BTreeMap<String, AlertRule>>,
    pub order: RefCell<Vec<String>>,
}

impl AlertRuleStore for MemoryAlertRuleStore {
    fn find(&self, code: &str) -> Option<AlertRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AlertRule) {
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

    fn list(&self) -> Vec<AlertRule> {
        let mut out: Vec<AlertRule> = Vec::new();
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

pub fn new_memory_alert_rule_store() -> Rc<dyn AlertRuleStore> {
    Rc::new(MemoryAlertRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAlertRuleStore {
    pub inner: Rc<dyn AlertRuleStore>,
    pub cache: RefCell<cache::TtlCache<AlertRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AlertRuleStore for CachedAlertRuleStore {
    fn find(&self, code: &str) -> Option<AlertRule> {
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

    fn save(&self, item: &AlertRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AlertRule> {
        self.inner.list()
    }
}

pub fn new_cached_alert_rule_store(
    inner: Rc<dyn AlertRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AlertRuleStore> {
    Rc::new(CachedAlertRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AlertRule>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects alert rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertRuleQuery {
    pub title: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AlertRuleQuery {
    pub fn matches(&self, item: &AlertRule) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AlertRule]) -> Vec<AlertRule> {
        let matched = collections::filter(items, |candidate: &AlertRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_alert_rules(store: Rc<dyn AlertRuleStore>, query: &AlertRuleQuery) -> Vec<AlertRule> {
    query.apply(&store.list())
}
