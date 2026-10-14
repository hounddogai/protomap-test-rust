use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps egress notices by code.
pub trait EgressNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EgressNotice>;
    fn save(&self, item: &EgressNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EgressNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEgressNoticeStore {
    pub items: RefCell<BTreeMap<String, EgressNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl EgressNoticeStore for MemoryEgressNoticeStore {
    fn find(&self, code: &str) -> Option<EgressNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EgressNotice) {
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

    fn list(&self) -> Vec<EgressNotice> {
        let mut out: Vec<EgressNotice> = Vec::new();
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

pub fn new_memory_egress_notice_store() -> Rc<dyn EgressNoticeStore> {
    Rc::new(MemoryEgressNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEgressNoticeStore {
    pub inner: Rc<dyn EgressNoticeStore>,
    pub cache: RefCell<cache::TtlCache<EgressNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EgressNoticeStore for CachedEgressNoticeStore {
    fn find(&self, code: &str) -> Option<EgressNotice> {
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

    fn save(&self, item: &EgressNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EgressNotice> {
        self.inner.list()
    }
}

pub fn new_cached_egress_notice_store(
    inner: Rc<dyn EgressNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EgressNoticeStore> {
    Rc::new(CachedEgressNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EgressNotice>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects egress notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressNoticeQuery {
    pub name: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EgressNoticeQuery {
    pub fn matches(&self, item: &EgressNotice) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EgressNotice]) -> Vec<EgressNotice> {
        let matched = collections::filter(items, |candidate: &EgressNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_egress_notices(store: Rc<dyn EgressNoticeStore>, query: &EgressNoticeQuery) -> Vec<EgressNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EgressNoticeStore> = new_memory_egress_notice_store();
        let item = new_egress_notice("code-968", "name-661");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
