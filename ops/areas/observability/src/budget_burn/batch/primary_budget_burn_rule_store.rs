use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary budget burn rules by code.
pub trait PrimaryBudgetBurnRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryBudgetBurnRule>;
    fn save(&self, item: &PrimaryBudgetBurnRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryBudgetBurnRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryBudgetBurnRuleStore {
    pub items: RefCell<BTreeMap<String, PrimaryBudgetBurnRule>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryBudgetBurnRuleStore for MemoryPrimaryBudgetBurnRuleStore {
    fn find(&self, code: &str) -> Option<PrimaryBudgetBurnRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryBudgetBurnRule) {
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

    fn list(&self) -> Vec<PrimaryBudgetBurnRule> {
        let mut out: Vec<PrimaryBudgetBurnRule> = Vec::new();
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

pub fn new_memory_primary_budget_burn_rule_store() -> Rc<dyn PrimaryBudgetBurnRuleStore> {
    Rc::new(MemoryPrimaryBudgetBurnRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryBudgetBurnRuleStore {
    pub inner: Rc<dyn PrimaryBudgetBurnRuleStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryBudgetBurnRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryBudgetBurnRuleStore for CachedPrimaryBudgetBurnRuleStore {
    fn find(&self, code: &str) -> Option<PrimaryBudgetBurnRule> {
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

    fn save(&self, item: &PrimaryBudgetBurnRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryBudgetBurnRule> {
        self.inner.list()
    }
}

pub fn new_cached_primary_budget_burn_rule_store(
    inner: Rc<dyn PrimaryBudgetBurnRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryBudgetBurnRuleStore> {
    Rc::new(CachedPrimaryBudgetBurnRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryBudgetBurnRule>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary budget burn rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnRuleQuery {
    pub namespace: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryBudgetBurnRuleQuery {
    pub fn matches(&self, item: &PrimaryBudgetBurnRule) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryBudgetBurnRule]) -> Vec<PrimaryBudgetBurnRule> {
        let matched = collections::filter(items, |candidate: &PrimaryBudgetBurnRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_budget_burn_rules(
    store: Rc<dyn PrimaryBudgetBurnRuleStore>,
    query: &PrimaryBudgetBurnRuleQuery,
) -> Vec<PrimaryBudgetBurnRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryBudgetBurnRuleStore> = new_memory_primary_budget_burn_rule_store();
        let item = new_primary_budget_burn_rule("code-728", "title-643");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
