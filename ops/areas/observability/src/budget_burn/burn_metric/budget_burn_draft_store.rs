use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps budget burn drafts by code.
pub trait BudgetBurnDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BudgetBurnDraft>;
    fn save(&self, item: &BudgetBurnDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BudgetBurnDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBudgetBurnDraftStore {
    pub items: RefCell<BTreeMap<String, BudgetBurnDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl BudgetBurnDraftStore for MemoryBudgetBurnDraftStore {
    fn find(&self, code: &str) -> Option<BudgetBurnDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BudgetBurnDraft) {
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

    fn list(&self) -> Vec<BudgetBurnDraft> {
        let mut out: Vec<BudgetBurnDraft> = Vec::new();
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

pub fn new_memory_budget_burn_draft_store() -> Rc<dyn BudgetBurnDraftStore> {
    Rc::new(MemoryBudgetBurnDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBudgetBurnDraftStore {
    pub inner: Rc<dyn BudgetBurnDraftStore>,
    pub cache: RefCell<cache::TtlCache<BudgetBurnDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BudgetBurnDraftStore for CachedBudgetBurnDraftStore {
    fn find(&self, code: &str) -> Option<BudgetBurnDraft> {
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

    fn save(&self, item: &BudgetBurnDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BudgetBurnDraft> {
        self.inner.list()
    }
}

pub fn new_cached_budget_burn_draft_store(
    inner: Rc<dyn BudgetBurnDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BudgetBurnDraftStore> {
    Rc::new(CachedBudgetBurnDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BudgetBurnDraft>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects budget burn drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDraftQuery {
    pub locale: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BudgetBurnDraftQuery {
    pub fn matches(&self, item: &BudgetBurnDraft) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
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

    pub fn apply(&self, items: &[BudgetBurnDraft]) -> Vec<BudgetBurnDraft> {
        let matched = collections::filter(items, |candidate: &BudgetBurnDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_budget_burn_drafts(
    store: Rc<dyn BudgetBurnDraftStore>,
    query: &BudgetBurnDraftQuery,
) -> Vec<BudgetBurnDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BudgetBurnDraftStore> = new_memory_budget_burn_draft_store();
        let item = new_budget_burn_draft("code-321", "name-819");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
