use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps allowlist records by code.
pub trait AllowlistRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AllowlistRecord>;
    fn save(&self, item: &AllowlistRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AllowlistRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAllowlistRecordStore {
    pub items: RefCell<BTreeMap<String, AllowlistRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl AllowlistRecordStore for MemoryAllowlistRecordStore {
    fn find(&self, code: &str) -> Option<AllowlistRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AllowlistRecord) {
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

    fn list(&self) -> Vec<AllowlistRecord> {
        let mut out: Vec<AllowlistRecord> = Vec::new();
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

pub fn new_memory_allowlist_record_store() -> Rc<dyn AllowlistRecordStore> {
    Rc::new(MemoryAllowlistRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAllowlistRecordStore {
    pub inner: Rc<dyn AllowlistRecordStore>,
    pub cache: RefCell<cache::TtlCache<AllowlistRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AllowlistRecordStore for CachedAllowlistRecordStore {
    fn find(&self, code: &str) -> Option<AllowlistRecord> {
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

    fn save(&self, item: &AllowlistRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AllowlistRecord> {
        self.inner.list()
    }
}

pub fn new_cached_allowlist_record_store(
    inner: Rc<dyn AllowlistRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AllowlistRecordStore> {
    Rc::new(CachedAllowlistRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AllowlistRecord>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects allowlist records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRecordQuery {
    pub channel: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AllowlistRecordQuery {
    pub fn matches(&self, item: &AllowlistRecord) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
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

    pub fn apply(&self, items: &[AllowlistRecord]) -> Vec<AllowlistRecord> {
        let matched = collections::filter(items, |candidate: &AllowlistRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_allowlist_records(
    store: Rc<dyn AllowlistRecordStore>,
    query: &AllowlistRecordQuery,
) -> Vec<AllowlistRecord> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AllowlistRecordStore> = new_memory_allowlist_record_store();
        let item = new_allowlist_record("code-295", "name-487");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
