use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient dns record baselines by code.
pub trait LenientDnsRecordBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientDnsRecordBaseline>;
    fn save(&self, item: &LenientDnsRecordBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientDnsRecordBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientDnsRecordBaselineStore {
    pub items: RefCell<BTreeMap<String, LenientDnsRecordBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientDnsRecordBaselineStore for MemoryLenientDnsRecordBaselineStore {
    fn find(&self, code: &str) -> Option<LenientDnsRecordBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientDnsRecordBaseline) {
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

    fn list(&self) -> Vec<LenientDnsRecordBaseline> {
        let mut out: Vec<LenientDnsRecordBaseline> = Vec::new();
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

pub fn new_memory_lenient_dns_record_baseline_store() -> Rc<dyn LenientDnsRecordBaselineStore> {
    Rc::new(MemoryLenientDnsRecordBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientDnsRecordBaselineStore {
    pub inner: Rc<dyn LenientDnsRecordBaselineStore>,
    pub cache: RefCell<cache::TtlCache<LenientDnsRecordBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientDnsRecordBaselineStore for CachedLenientDnsRecordBaselineStore {
    fn find(&self, code: &str) -> Option<LenientDnsRecordBaseline> {
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

    fn save(&self, item: &LenientDnsRecordBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientDnsRecordBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_dns_record_baseline_store(
    inner: Rc<dyn LenientDnsRecordBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientDnsRecordBaselineStore> {
    Rc::new(CachedLenientDnsRecordBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientDnsRecordBaseline>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient dns record baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordBaselineQuery {
    pub label: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientDnsRecordBaselineQuery {
    pub fn matches(&self, item: &LenientDnsRecordBaseline) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientDnsRecordBaseline]) -> Vec<LenientDnsRecordBaseline> {
        let matched = collections::filter(items, |candidate: &LenientDnsRecordBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_dns_record_baselines(
    store: Rc<dyn LenientDnsRecordBaselineStore>,
    query: &LenientDnsRecordBaselineQuery,
) -> Vec<LenientDnsRecordBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientDnsRecordBaselineStore> = new_memory_lenient_dns_record_baseline_store();
        let item = new_lenient_dns_record_baseline("code-589", "label-957");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
