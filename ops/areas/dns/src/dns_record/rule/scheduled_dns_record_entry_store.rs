use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled dns record entries by code.
pub trait ScheduledDnsRecordEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledDnsRecordEntry>;
    fn save(&self, item: &ScheduledDnsRecordEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledDnsRecordEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledDnsRecordEntryStore {
    pub items: RefCell<BTreeMap<String, ScheduledDnsRecordEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledDnsRecordEntryStore for MemoryScheduledDnsRecordEntryStore {
    fn find(&self, code: &str) -> Option<ScheduledDnsRecordEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledDnsRecordEntry) {
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

    fn list(&self) -> Vec<ScheduledDnsRecordEntry> {
        let mut out: Vec<ScheduledDnsRecordEntry> = Vec::new();
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

pub fn new_memory_scheduled_dns_record_entry_store() -> Rc<dyn ScheduledDnsRecordEntryStore> {
    Rc::new(MemoryScheduledDnsRecordEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledDnsRecordEntryStore {
    pub inner: Rc<dyn ScheduledDnsRecordEntryStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledDnsRecordEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledDnsRecordEntryStore for CachedScheduledDnsRecordEntryStore {
    fn find(&self, code: &str) -> Option<ScheduledDnsRecordEntry> {
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

    fn save(&self, item: &ScheduledDnsRecordEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledDnsRecordEntry> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_dns_record_entry_store(
    inner: Rc<dyn ScheduledDnsRecordEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledDnsRecordEntryStore> {
    Rc::new(CachedScheduledDnsRecordEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledDnsRecordEntry>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled dns record entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDnsRecordEntryQuery {
    pub name: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledDnsRecordEntryQuery {
    pub fn matches(&self, item: &ScheduledDnsRecordEntry) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[ScheduledDnsRecordEntry]) -> Vec<ScheduledDnsRecordEntry> {
        let matched = collections::filter(items, |candidate: &ScheduledDnsRecordEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_dns_record_entries(
    store: Rc<dyn ScheduledDnsRecordEntryStore>,
    query: &ScheduledDnsRecordEntryQuery,
) -> Vec<ScheduledDnsRecordEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledDnsRecordEntryStore> = new_memory_scheduled_dns_record_entry_store();
        let item = new_scheduled_dns_record_entry("code-270", "name-131");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
