use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled probe rules by code.
pub trait ScheduledProbeRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledProbeRule>;
    fn save(&self, item: &ScheduledProbeRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledProbeRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledProbeRuleStore {
    pub items: RefCell<BTreeMap<String, ScheduledProbeRule>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledProbeRuleStore for MemoryScheduledProbeRuleStore {
    fn find(&self, code: &str) -> Option<ScheduledProbeRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledProbeRule) {
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

    fn list(&self) -> Vec<ScheduledProbeRule> {
        let mut out: Vec<ScheduledProbeRule> = Vec::new();
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

pub fn new_memory_scheduled_probe_rule_store() -> Rc<dyn ScheduledProbeRuleStore> {
    Rc::new(MemoryScheduledProbeRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledProbeRuleStore {
    pub inner: Rc<dyn ScheduledProbeRuleStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledProbeRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledProbeRuleStore for CachedScheduledProbeRuleStore {
    fn find(&self, code: &str) -> Option<ScheduledProbeRule> {
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

    fn save(&self, item: &ScheduledProbeRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledProbeRule> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_probe_rule_store(
    inner: Rc<dyn ScheduledProbeRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledProbeRuleStore> {
    Rc::new(CachedScheduledProbeRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledProbeRule>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled probe rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeRuleQuery {
    pub namespace: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledProbeRuleQuery {
    pub fn matches(&self, item: &ScheduledProbeRule) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[ScheduledProbeRule]) -> Vec<ScheduledProbeRule> {
        let matched = collections::filter(items, |candidate: &ScheduledProbeRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_probe_rules(
    store: Rc<dyn ScheduledProbeRuleStore>,
    query: &ScheduledProbeRuleQuery,
) -> Vec<ScheduledProbeRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledProbeRuleStore> = new_memory_scheduled_probe_rule_store();
        let item = new_scheduled_probe_rule("code-363", "name-543");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
