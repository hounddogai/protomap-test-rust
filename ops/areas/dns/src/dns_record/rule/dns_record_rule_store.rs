use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dns record rules by code.
pub trait DnsRecordRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DnsRecordRule>;
    fn save(&self, item: &DnsRecordRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DnsRecordRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDnsRecordRuleStore {
    pub items: RefCell<BTreeMap<String, DnsRecordRule>>,
    pub order: RefCell<Vec<String>>,
}

impl DnsRecordRuleStore for MemoryDnsRecordRuleStore {
    fn find(&self, code: &str) -> Option<DnsRecordRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DnsRecordRule) {
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

    fn list(&self) -> Vec<DnsRecordRule> {
        let mut out: Vec<DnsRecordRule> = Vec::new();
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

pub fn new_memory_dns_record_rule_store() -> Rc<dyn DnsRecordRuleStore> {
    Rc::new(MemoryDnsRecordRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDnsRecordRuleStore {
    pub inner: Rc<dyn DnsRecordRuleStore>,
    pub cache: RefCell<cache::TtlCache<DnsRecordRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DnsRecordRuleStore for CachedDnsRecordRuleStore {
    fn find(&self, code: &str) -> Option<DnsRecordRule> {
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

    fn save(&self, item: &DnsRecordRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DnsRecordRule> {
        self.inner.list()
    }
}

pub fn new_cached_dns_record_rule_store(
    inner: Rc<dyn DnsRecordRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DnsRecordRuleStore> {
    Rc::new(CachedDnsRecordRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DnsRecordRule>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dns record rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRuleQuery {
    pub name: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DnsRecordRuleQuery {
    pub fn matches(&self, item: &DnsRecordRule) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DnsRecordRule]) -> Vec<DnsRecordRule> {
        let matched = collections::filter(items, |candidate: &DnsRecordRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dns_record_rules(store: Rc<dyn DnsRecordRuleStore>, query: &DnsRecordRuleQuery) -> Vec<DnsRecordRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DnsRecordRuleStore> = new_memory_dns_record_rule_store();
        let item = new_dns_record_rule("code-110", "name-863");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
