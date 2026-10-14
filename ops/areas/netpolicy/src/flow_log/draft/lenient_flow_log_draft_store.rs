use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient flow log drafts by code.
pub trait LenientFlowLogDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientFlowLogDraft>;
    fn save(&self, item: &LenientFlowLogDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientFlowLogDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientFlowLogDraftStore {
    pub items: RefCell<BTreeMap<String, LenientFlowLogDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientFlowLogDraftStore for MemoryLenientFlowLogDraftStore {
    fn find(&self, code: &str) -> Option<LenientFlowLogDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientFlowLogDraft) {
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

    fn list(&self) -> Vec<LenientFlowLogDraft> {
        let mut out: Vec<LenientFlowLogDraft> = Vec::new();
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

pub fn new_memory_lenient_flow_log_draft_store() -> Rc<dyn LenientFlowLogDraftStore> {
    Rc::new(MemoryLenientFlowLogDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientFlowLogDraftStore {
    pub inner: Rc<dyn LenientFlowLogDraftStore>,
    pub cache: RefCell<cache::TtlCache<LenientFlowLogDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientFlowLogDraftStore for CachedLenientFlowLogDraftStore {
    fn find(&self, code: &str) -> Option<LenientFlowLogDraft> {
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

    fn save(&self, item: &LenientFlowLogDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientFlowLogDraft> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_flow_log_draft_store(
    inner: Rc<dyn LenientFlowLogDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientFlowLogDraftStore> {
    Rc::new(CachedLenientFlowLogDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientFlowLogDraft>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient flow log drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogDraftQuery {
    pub name: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientFlowLogDraftQuery {
    pub fn matches(&self, item: &LenientFlowLogDraft) -> bool {
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

    pub fn apply(&self, items: &[LenientFlowLogDraft]) -> Vec<LenientFlowLogDraft> {
        let matched = collections::filter(items, |candidate: &LenientFlowLogDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_flow_log_drafts(
    store: Rc<dyn LenientFlowLogDraftStore>,
    query: &LenientFlowLogDraftQuery,
) -> Vec<LenientFlowLogDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientFlowLogDraftStore> = new_memory_lenient_flow_log_draft_store();
        let item = new_lenient_flow_log_draft("code-463", "name-587");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
