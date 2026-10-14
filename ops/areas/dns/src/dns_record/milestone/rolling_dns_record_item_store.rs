use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling dns record items by code.
pub trait RollingDnsRecordItemStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingDnsRecordItem>;
    fn save(&self, item: &RollingDnsRecordItem);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingDnsRecordItem>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingDnsRecordItemStore {
    pub items: RefCell<BTreeMap<String, RollingDnsRecordItem>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingDnsRecordItemStore for MemoryRollingDnsRecordItemStore {
    fn find(&self, code: &str) -> Option<RollingDnsRecordItem> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingDnsRecordItem) {
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

    fn list(&self) -> Vec<RollingDnsRecordItem> {
        let mut out: Vec<RollingDnsRecordItem> = Vec::new();
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

pub fn new_memory_rolling_dns_record_item_store() -> Rc<dyn RollingDnsRecordItemStore> {
    Rc::new(MemoryRollingDnsRecordItemStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingDnsRecordItemStore {
    pub inner: Rc<dyn RollingDnsRecordItemStore>,
    pub cache: RefCell<cache::TtlCache<RollingDnsRecordItem>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingDnsRecordItemStore for CachedRollingDnsRecordItemStore {
    fn find(&self, code: &str) -> Option<RollingDnsRecordItem> {
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

    fn save(&self, item: &RollingDnsRecordItem) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingDnsRecordItem> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_dns_record_item_store(
    inner: Rc<dyn RollingDnsRecordItemStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingDnsRecordItemStore> {
    Rc::new(CachedRollingDnsRecordItemStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingDnsRecordItem>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling dns record items. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDnsRecordItemQuery {
    pub label: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingDnsRecordItemQuery {
    pub fn matches(&self, item: &RollingDnsRecordItem) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingDnsRecordItem]) -> Vec<RollingDnsRecordItem> {
        let matched = collections::filter(items, |candidate: &RollingDnsRecordItem| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_dns_record_items(
    store: Rc<dyn RollingDnsRecordItemStore>,
    query: &RollingDnsRecordItemQuery,
) -> Vec<RollingDnsRecordItem> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingDnsRecordItemStore> = new_memory_rolling_dns_record_item_store();
        let item = new_rolling_dns_record_item("code-27", "label-353");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
