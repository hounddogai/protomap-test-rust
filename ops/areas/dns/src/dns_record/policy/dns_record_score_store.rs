use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dns record scores by code.
pub trait DnsRecordScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DnsRecordScore>;
    fn save(&self, item: &DnsRecordScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DnsRecordScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDnsRecordScoreStore {
    pub items: RefCell<BTreeMap<String, DnsRecordScore>>,
    pub order: RefCell<Vec<String>>,
}

impl DnsRecordScoreStore for MemoryDnsRecordScoreStore {
    fn find(&self, code: &str) -> Option<DnsRecordScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DnsRecordScore) {
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

    fn list(&self) -> Vec<DnsRecordScore> {
        let mut out: Vec<DnsRecordScore> = Vec::new();
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

pub fn new_memory_dns_record_score_store() -> Rc<dyn DnsRecordScoreStore> {
    Rc::new(MemoryDnsRecordScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDnsRecordScoreStore {
    pub inner: Rc<dyn DnsRecordScoreStore>,
    pub cache: RefCell<cache::TtlCache<DnsRecordScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DnsRecordScoreStore for CachedDnsRecordScoreStore {
    fn find(&self, code: &str) -> Option<DnsRecordScore> {
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

    fn save(&self, item: &DnsRecordScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DnsRecordScore> {
        self.inner.list()
    }
}

pub fn new_cached_dns_record_score_store(
    inner: Rc<dyn DnsRecordScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DnsRecordScoreStore> {
    Rc::new(CachedDnsRecordScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DnsRecordScore>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dns record scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordScoreQuery {
    pub scope: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DnsRecordScoreQuery {
    pub fn matches(&self, item: &DnsRecordScore) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DnsRecordScore]) -> Vec<DnsRecordScore> {
        let matched = collections::filter(items, |candidate: &DnsRecordScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dns_record_scores(store: Rc<dyn DnsRecordScoreStore>, query: &DnsRecordScoreQuery) -> Vec<DnsRecordScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DnsRecordScoreStore> = new_memory_dns_record_score_store();
        let item = new_dns_record_score("code-195", "name-403");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
