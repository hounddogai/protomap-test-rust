use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dns record policies by code.
pub trait DnsRecordPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DnsRecordPolicy>;
    fn save(&self, item: &DnsRecordPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DnsRecordPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDnsRecordPolicyStore {
    pub items: RefCell<BTreeMap<String, DnsRecordPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl DnsRecordPolicyStore for MemoryDnsRecordPolicyStore {
    fn find(&self, code: &str) -> Option<DnsRecordPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DnsRecordPolicy) {
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

    fn list(&self) -> Vec<DnsRecordPolicy> {
        let mut out: Vec<DnsRecordPolicy> = Vec::new();
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

pub fn new_memory_dns_record_policy_store() -> Rc<dyn DnsRecordPolicyStore> {
    Rc::new(MemoryDnsRecordPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDnsRecordPolicyStore {
    pub inner: Rc<dyn DnsRecordPolicyStore>,
    pub cache: RefCell<cache::TtlCache<DnsRecordPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DnsRecordPolicyStore for CachedDnsRecordPolicyStore {
    fn find(&self, code: &str) -> Option<DnsRecordPolicy> {
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

    fn save(&self, item: &DnsRecordPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DnsRecordPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_dns_record_policy_store(
    inner: Rc<dyn DnsRecordPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DnsRecordPolicyStore> {
    Rc::new(CachedDnsRecordPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DnsRecordPolicy>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dns record policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPolicyQuery {
    pub namespace: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DnsRecordPolicyQuery {
    pub fn matches(&self, item: &DnsRecordPolicy) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DnsRecordPolicy]) -> Vec<DnsRecordPolicy> {
        let matched = collections::filter(items, |candidate: &DnsRecordPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dns_record_policies(
    store: Rc<dyn DnsRecordPolicyStore>,
    query: &DnsRecordPolicyQuery,
) -> Vec<DnsRecordPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DnsRecordPolicyStore> = new_memory_dns_record_policy_store();
        let item = new_dns_record_policy("code-803", "name-70");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
