use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled health check rules by code.
pub trait ScheduledHealthCheckRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledHealthCheckRule>;
    fn save(&self, item: &ScheduledHealthCheckRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledHealthCheckRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledHealthCheckRuleStore {
    pub items: RefCell<BTreeMap<String, ScheduledHealthCheckRule>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledHealthCheckRuleStore for MemoryScheduledHealthCheckRuleStore {
    fn find(&self, code: &str) -> Option<ScheduledHealthCheckRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledHealthCheckRule) {
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

    fn list(&self) -> Vec<ScheduledHealthCheckRule> {
        let mut out: Vec<ScheduledHealthCheckRule> = Vec::new();
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

pub fn new_memory_scheduled_health_check_rule_store() -> Rc<dyn ScheduledHealthCheckRuleStore> {
    Rc::new(MemoryScheduledHealthCheckRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledHealthCheckRuleStore {
    pub inner: Rc<dyn ScheduledHealthCheckRuleStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledHealthCheckRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledHealthCheckRuleStore for CachedScheduledHealthCheckRuleStore {
    fn find(&self, code: &str) -> Option<ScheduledHealthCheckRule> {
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

    fn save(&self, item: &ScheduledHealthCheckRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledHealthCheckRule> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_health_check_rule_store(
    inner: Rc<dyn ScheduledHealthCheckRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledHealthCheckRuleStore> {
    Rc::new(CachedScheduledHealthCheckRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledHealthCheckRule>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled health check rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckRuleQuery {
    pub segment: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledHealthCheckRuleQuery {
    pub fn matches(&self, item: &ScheduledHealthCheckRule) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[ScheduledHealthCheckRule]) -> Vec<ScheduledHealthCheckRule> {
        let matched = collections::filter(items, |candidate: &ScheduledHealthCheckRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_health_check_rules(
    store: Rc<dyn ScheduledHealthCheckRuleStore>,
    query: &ScheduledHealthCheckRuleQuery,
) -> Vec<ScheduledHealthCheckRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledHealthCheckRuleStore> = new_memory_scheduled_health_check_rule_store();
        let item = new_scheduled_health_check_rule("code-985", "title-321");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
