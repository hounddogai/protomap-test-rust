use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered plan drafts by code.
pub trait TieredPlanDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredPlanDraft>;
    fn save(&self, item: &TieredPlanDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredPlanDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredPlanDraftStore {
    pub items: RefCell<BTreeMap<String, TieredPlanDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredPlanDraftStore for MemoryTieredPlanDraftStore {
    fn find(&self, code: &str) -> Option<TieredPlanDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredPlanDraft) {
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

    fn list(&self) -> Vec<TieredPlanDraft> {
        let mut out: Vec<TieredPlanDraft> = Vec::new();
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

pub fn new_memory_tiered_plan_draft_store() -> Rc<dyn TieredPlanDraftStore> {
    Rc::new(MemoryTieredPlanDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredPlanDraftStore {
    pub inner: Rc<dyn TieredPlanDraftStore>,
    pub cache: RefCell<cache::TtlCache<TieredPlanDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredPlanDraftStore for CachedTieredPlanDraftStore {
    fn find(&self, code: &str) -> Option<TieredPlanDraft> {
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

    fn save(&self, item: &TieredPlanDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredPlanDraft> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_plan_draft_store(
    inner: Rc<dyn TieredPlanDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredPlanDraftStore> {
    Rc::new(CachedTieredPlanDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredPlanDraft>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered plan drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanDraftQuery {
    pub region: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredPlanDraftQuery {
    pub fn matches(&self, item: &TieredPlanDraft) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredPlanDraft]) -> Vec<TieredPlanDraft> {
        let matched = collections::filter(items, |candidate: &TieredPlanDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_plan_drafts(
    store: Rc<dyn TieredPlanDraftStore>,
    query: &TieredPlanDraftQuery,
) -> Vec<TieredPlanDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredPlanDraftStore> = new_memory_tiered_plan_draft_store();
        let item = new_tiered_plan_draft("code-540", "label-171");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
