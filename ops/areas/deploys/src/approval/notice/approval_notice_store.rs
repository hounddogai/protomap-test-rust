use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval notices by code.
pub trait ApprovalNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalNotice>;
    fn save(&self, item: &ApprovalNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalNoticeStore {
    pub items: RefCell<BTreeMap<String, ApprovalNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalNoticeStore for MemoryApprovalNoticeStore {
    fn find(&self, code: &str) -> Option<ApprovalNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalNotice) {
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

    fn list(&self) -> Vec<ApprovalNotice> {
        let mut out: Vec<ApprovalNotice> = Vec::new();
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

pub fn new_memory_approval_notice_store() -> Rc<dyn ApprovalNoticeStore> {
    Rc::new(MemoryApprovalNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalNoticeStore {
    pub inner: Rc<dyn ApprovalNoticeStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalNoticeStore for CachedApprovalNoticeStore {
    fn find(&self, code: &str) -> Option<ApprovalNotice> {
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

    fn save(&self, item: &ApprovalNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalNotice> {
        self.inner.list()
    }
}

pub fn new_cached_approval_notice_store(
    inner: Rc<dyn ApprovalNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalNoticeStore> {
    Rc::new(CachedApprovalNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalNotice>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalNoticeQuery {
    pub label: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalNoticeQuery {
    pub fn matches(&self, item: &ApprovalNotice) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ApprovalNotice]) -> Vec<ApprovalNotice> {
        let matched = collections::filter(items, |candidate: &ApprovalNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_notices(store: Rc<dyn ApprovalNoticeStore>, query: &ApprovalNoticeQuery) -> Vec<ApprovalNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ApprovalNoticeStore> = new_memory_approval_notice_store();
        let item = new_approval_notice("code-515", "label-653");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
