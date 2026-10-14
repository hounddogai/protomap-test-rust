use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional reservation items by code.
pub trait RegionalReservationItemStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalReservationItem>;
    fn save(&self, item: &RegionalReservationItem);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalReservationItem>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalReservationItemStore {
    pub items: RefCell<BTreeMap<String, RegionalReservationItem>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalReservationItemStore for MemoryRegionalReservationItemStore {
    fn find(&self, code: &str) -> Option<RegionalReservationItem> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalReservationItem) {
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

    fn list(&self) -> Vec<RegionalReservationItem> {
        let mut out: Vec<RegionalReservationItem> = Vec::new();
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

pub fn new_memory_regional_reservation_item_store() -> Rc<dyn RegionalReservationItemStore> {
    Rc::new(MemoryRegionalReservationItemStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalReservationItemStore {
    pub inner: Rc<dyn RegionalReservationItemStore>,
    pub cache: RefCell<cache::TtlCache<RegionalReservationItem>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalReservationItemStore for CachedRegionalReservationItemStore {
    fn find(&self, code: &str) -> Option<RegionalReservationItem> {
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

    fn save(&self, item: &RegionalReservationItem) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalReservationItem> {
        self.inner.list()
    }
}

pub fn new_cached_regional_reservation_item_store(
    inner: Rc<dyn RegionalReservationItemStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalReservationItemStore> {
    Rc::new(CachedRegionalReservationItemStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalReservationItem>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional reservation items. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationItemQuery {
    pub segment: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalReservationItemQuery {
    pub fn matches(&self, item: &RegionalReservationItem) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[RegionalReservationItem]) -> Vec<RegionalReservationItem> {
        let matched = collections::filter(items, |candidate: &RegionalReservationItem| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_reservation_items(
    store: Rc<dyn RegionalReservationItemStore>,
    query: &RegionalReservationItemQuery,
) -> Vec<RegionalReservationItem> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalReservationItemStore> = new_memory_regional_reservation_item_store();
        let item = new_regional_reservation_item("code-222", "title-744");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
