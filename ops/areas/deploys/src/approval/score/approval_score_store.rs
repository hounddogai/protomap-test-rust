use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval scores by code.
pub trait ApprovalScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalScore>;
    fn save(&self, item: &ApprovalScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalScoreStore {
    pub items: RefCell<BTreeMap<String, ApprovalScore>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalScoreStore for MemoryApprovalScoreStore {
    fn find(&self, code: &str) -> Option<ApprovalScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalScore) {
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

    fn list(&self) -> Vec<ApprovalScore> {
        let mut out: Vec<ApprovalScore> = Vec::new();
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

pub fn new_memory_approval_score_store() -> Rc<dyn ApprovalScoreStore> {
    Rc::new(MemoryApprovalScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalScoreStore {
    pub inner: Rc<dyn ApprovalScoreStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalScoreStore for CachedApprovalScoreStore {
    fn find(&self, code: &str) -> Option<ApprovalScore> {
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

    fn save(&self, item: &ApprovalScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalScore> {
        self.inner.list()
    }
}

pub fn new_cached_approval_score_store(
    inner: Rc<dyn ApprovalScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalScoreStore> {
    Rc::new(CachedApprovalScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalScore>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalScoreQuery {
    pub label: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalScoreQuery {
    pub fn matches(&self, item: &ApprovalScore) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ApprovalScore]) -> Vec<ApprovalScore> {
        let matched = collections::filter(items, |candidate: &ApprovalScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_scores(store: Rc<dyn ApprovalScoreStore>, query: &ApprovalScoreQuery) -> Vec<ApprovalScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ApprovalScoreStore> = new_memory_approval_score_store();
        let item = new_approval_score("code-990", "label-893");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
