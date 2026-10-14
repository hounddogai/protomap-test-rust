use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps escalation plans by code.
pub trait EscalationPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EscalationPlan>;
    fn save(&self, item: &EscalationPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EscalationPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEscalationPlanStore {
    pub items: RefCell<BTreeMap<String, EscalationPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl EscalationPlanStore for MemoryEscalationPlanStore {
    fn find(&self, code: &str) -> Option<EscalationPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EscalationPlan) {
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

    fn list(&self) -> Vec<EscalationPlan> {
        let mut out: Vec<EscalationPlan> = Vec::new();
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

pub fn new_memory_escalation_plan_store() -> Rc<dyn EscalationPlanStore> {
    Rc::new(MemoryEscalationPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEscalationPlanStore {
    pub inner: Rc<dyn EscalationPlanStore>,
    pub cache: RefCell<cache::TtlCache<EscalationPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EscalationPlanStore for CachedEscalationPlanStore {
    fn find(&self, code: &str) -> Option<EscalationPlan> {
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

    fn save(&self, item: &EscalationPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EscalationPlan> {
        self.inner.list()
    }
}

pub fn new_cached_escalation_plan_store(
    inner: Rc<dyn EscalationPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EscalationPlanStore> {
    Rc::new(CachedEscalationPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EscalationPlan>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects escalation plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPlanQuery {
    pub name: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EscalationPlanQuery {
    pub fn matches(&self, item: &EscalationPlan) -> bool {
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

    pub fn apply(&self, items: &[EscalationPlan]) -> Vec<EscalationPlan> {
        let matched = collections::filter(items, |candidate: &EscalationPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_escalation_plans(store: Rc<dyn EscalationPlanStore>, query: &EscalationPlanQuery) -> Vec<EscalationPlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EscalationPlanStore> = new_memory_escalation_plan_store();
        let item = new_escalation_plan("code-552", "name-598");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
