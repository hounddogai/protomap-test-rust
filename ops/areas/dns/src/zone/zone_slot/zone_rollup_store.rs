use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps zone rollups by code.
pub trait ZoneRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ZoneRollup>;
    fn save(&self, item: &ZoneRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ZoneRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryZoneRollupStore {
    pub items: RefCell<BTreeMap<String, ZoneRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl ZoneRollupStore for MemoryZoneRollupStore {
    fn find(&self, code: &str) -> Option<ZoneRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ZoneRollup) {
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

    fn list(&self) -> Vec<ZoneRollup> {
        let mut out: Vec<ZoneRollup> = Vec::new();
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

pub fn new_memory_zone_rollup_store() -> Rc<dyn ZoneRollupStore> {
    Rc::new(MemoryZoneRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedZoneRollupStore {
    pub inner: Rc<dyn ZoneRollupStore>,
    pub cache: RefCell<cache::TtlCache<ZoneRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ZoneRollupStore for CachedZoneRollupStore {
    fn find(&self, code: &str) -> Option<ZoneRollup> {
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

    fn save(&self, item: &ZoneRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ZoneRollup> {
        self.inner.list()
    }
}

pub fn new_cached_zone_rollup_store(
    inner: Rc<dyn ZoneRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ZoneRollupStore> {
    Rc::new(CachedZoneRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ZoneRollup>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects zone rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRollupQuery {
    pub title: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ZoneRollupQuery {
    pub fn matches(&self, item: &ZoneRollup) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ZoneRollup]) -> Vec<ZoneRollup> {
        let matched = collections::filter(items, |candidate: &ZoneRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_zone_rollups(store: Rc<dyn ZoneRollupStore>, query: &ZoneRollupQuery) -> Vec<ZoneRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ZoneRollupStore> = new_memory_zone_rollup_store();
        let item = new_zone_rollup("code-861", "title-564");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
