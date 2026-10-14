use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps silence budgets by code.
pub trait SilenceBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SilenceBudget>;
    fn save(&self, item: &SilenceBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SilenceBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySilenceBudgetStore {
    pub items: RefCell<BTreeMap<String, SilenceBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl SilenceBudgetStore for MemorySilenceBudgetStore {
    fn find(&self, code: &str) -> Option<SilenceBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SilenceBudget) {
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

    fn list(&self) -> Vec<SilenceBudget> {
        let mut out: Vec<SilenceBudget> = Vec::new();
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

pub fn new_memory_silence_budget_store() -> Rc<dyn SilenceBudgetStore> {
    Rc::new(MemorySilenceBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSilenceBudgetStore {
    pub inner: Rc<dyn SilenceBudgetStore>,
    pub cache: RefCell<cache::TtlCache<SilenceBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SilenceBudgetStore for CachedSilenceBudgetStore {
    fn find(&self, code: &str) -> Option<SilenceBudget> {
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

    fn save(&self, item: &SilenceBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SilenceBudget> {
        self.inner.list()
    }
}

pub fn new_cached_silence_budget_store(
    inner: Rc<dyn SilenceBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SilenceBudgetStore> {
    Rc::new(CachedSilenceBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SilenceBudget>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects silence budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceBudgetQuery {
    pub title: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SilenceBudgetQuery {
    pub fn matches(&self, item: &SilenceBudget) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SilenceBudget]) -> Vec<SilenceBudget> {
        let matched = collections::filter(items, |candidate: &SilenceBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_silence_budgets(store: Rc<dyn SilenceBudgetStore>, query: &SilenceBudgetQuery) -> Vec<SilenceBudget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SilenceBudgetStore> = new_memory_silence_budget_store();
        let item = new_silence_budget("code-225", "title-789");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
