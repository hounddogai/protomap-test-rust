use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps slo profiles by code.
pub trait SloProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SloProfile>;
    fn save(&self, item: &SloProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SloProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySloProfileStore {
    pub items: RefCell<BTreeMap<String, SloProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl SloProfileStore for MemorySloProfileStore {
    fn find(&self, code: &str) -> Option<SloProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SloProfile) {
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

    fn list(&self) -> Vec<SloProfile> {
        let mut out: Vec<SloProfile> = Vec::new();
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

pub fn new_memory_slo_profile_store() -> Rc<dyn SloProfileStore> {
    Rc::new(MemorySloProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSloProfileStore {
    pub inner: Rc<dyn SloProfileStore>,
    pub cache: RefCell<cache::TtlCache<SloProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SloProfileStore for CachedSloProfileStore {
    fn find(&self, code: &str) -> Option<SloProfile> {
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

    fn save(&self, item: &SloProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SloProfile> {
        self.inner.list()
    }
}

pub fn new_cached_slo_profile_store(
    inner: Rc<dyn SloProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SloProfileStore> {
    Rc::new(CachedSloProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SloProfile>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects slo profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SloProfileQuery {
    pub segment: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SloProfileQuery {
    pub fn matches(&self, item: &SloProfile) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SloProfile]) -> Vec<SloProfile> {
        let matched = collections::filter(items, |candidate: &SloProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_slo_profiles(store: Rc<dyn SloProfileStore>, query: &SloProfileQuery) -> Vec<SloProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SloProfileStore> = new_memory_slo_profile_store();
        let item = new_slo_profile("code-164", "label-379");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
