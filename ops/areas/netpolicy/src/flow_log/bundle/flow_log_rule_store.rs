use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps flow log rules by code.
pub trait FlowLogRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FlowLogRule>;
    fn save(&self, item: &FlowLogRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FlowLogRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFlowLogRuleStore {
    pub items: RefCell<BTreeMap<String, FlowLogRule>>,
    pub order: RefCell<Vec<String>>,
}

impl FlowLogRuleStore for MemoryFlowLogRuleStore {
    fn find(&self, code: &str) -> Option<FlowLogRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FlowLogRule) {
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

    fn list(&self) -> Vec<FlowLogRule> {
        let mut out: Vec<FlowLogRule> = Vec::new();
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

pub fn new_memory_flow_log_rule_store() -> Rc<dyn FlowLogRuleStore> {
    Rc::new(MemoryFlowLogRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFlowLogRuleStore {
    pub inner: Rc<dyn FlowLogRuleStore>,
    pub cache: RefCell<cache::TtlCache<FlowLogRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FlowLogRuleStore for CachedFlowLogRuleStore {
    fn find(&self, code: &str) -> Option<FlowLogRule> {
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

    fn save(&self, item: &FlowLogRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FlowLogRule> {
        self.inner.list()
    }
}

pub fn new_cached_flow_log_rule_store(
    inner: Rc<dyn FlowLogRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FlowLogRuleStore> {
    Rc::new(CachedFlowLogRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FlowLogRule>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects flow log rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRuleQuery {
    pub name: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FlowLogRuleQuery {
    pub fn matches(&self, item: &FlowLogRule) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FlowLogRule]) -> Vec<FlowLogRule> {
        let matched = collections::filter(items, |candidate: &FlowLogRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_flow_log_rules(store: Rc<dyn FlowLogRuleStore>, query: &FlowLogRuleQuery) -> Vec<FlowLogRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FlowLogRuleStore> = new_memory_flow_log_rule_store();
        let item = new_flow_log_rule("code-208", "name-362");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
