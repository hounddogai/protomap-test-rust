use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient escalation rules by code.
pub trait LenientEscalationRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientEscalationRule>;
    fn save(&self, item: &LenientEscalationRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientEscalationRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientEscalationRuleStore {
    pub items: RefCell<BTreeMap<String, LenientEscalationRule>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientEscalationRuleStore for MemoryLenientEscalationRuleStore {
    fn find(&self, code: &str) -> Option<LenientEscalationRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientEscalationRule) {
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

    fn list(&self) -> Vec<LenientEscalationRule> {
        let mut out: Vec<LenientEscalationRule> = Vec::new();
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

pub fn new_memory_lenient_escalation_rule_store() -> Rc<dyn LenientEscalationRuleStore> {
    Rc::new(MemoryLenientEscalationRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientEscalationRuleStore {
    pub inner: Rc<dyn LenientEscalationRuleStore>,
    pub cache: RefCell<cache::TtlCache<LenientEscalationRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientEscalationRuleStore for CachedLenientEscalationRuleStore {
    fn find(&self, code: &str) -> Option<LenientEscalationRule> {
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

    fn save(&self, item: &LenientEscalationRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientEscalationRule> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_escalation_rule_store(
    inner: Rc<dyn LenientEscalationRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientEscalationRuleStore> {
    Rc::new(CachedLenientEscalationRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientEscalationRule>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient escalation rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEscalationRuleQuery {
    pub segment: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientEscalationRuleQuery {
    pub fn matches(&self, item: &LenientEscalationRule) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientEscalationRule]) -> Vec<LenientEscalationRule> {
        let matched = collections::filter(items, |candidate: &LenientEscalationRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_escalation_rules(
    store: Rc<dyn LenientEscalationRuleStore>,
    query: &LenientEscalationRuleQuery,
) -> Vec<LenientEscalationRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientEscalationRuleStore> = new_memory_lenient_escalation_rule_store();
        let item = new_lenient_escalation_rule("code-882", "name-228");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
