use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered reservation scores by code.
pub trait TieredReservationScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredReservationScore>;
    fn save(&self, item: &TieredReservationScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredReservationScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredReservationScoreStore {
    pub items: RefCell<BTreeMap<String, TieredReservationScore>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredReservationScoreStore for MemoryTieredReservationScoreStore {
    fn find(&self, code: &str) -> Option<TieredReservationScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredReservationScore) {
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

    fn list(&self) -> Vec<TieredReservationScore> {
        let mut out: Vec<TieredReservationScore> = Vec::new();
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

pub fn new_memory_tiered_reservation_score_store() -> Rc<dyn TieredReservationScoreStore> {
    Rc::new(MemoryTieredReservationScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredReservationScoreStore {
    pub inner: Rc<dyn TieredReservationScoreStore>,
    pub cache: RefCell<cache::TtlCache<TieredReservationScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredReservationScoreStore for CachedTieredReservationScoreStore {
    fn find(&self, code: &str) -> Option<TieredReservationScore> {
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

    fn save(&self, item: &TieredReservationScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredReservationScore> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_reservation_score_store(
    inner: Rc<dyn TieredReservationScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredReservationScoreStore> {
    Rc::new(CachedTieredReservationScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredReservationScore>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered reservation scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationScoreQuery {
    pub source: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredReservationScoreQuery {
    pub fn matches(&self, item: &TieredReservationScore) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredReservationScore]) -> Vec<TieredReservationScore> {
        let matched = collections::filter(items, |candidate: &TieredReservationScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_reservation_scores(
    store: Rc<dyn TieredReservationScoreStore>,
    query: &TieredReservationScoreQuery,
) -> Vec<TieredReservationScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredReservationScoreStore> = new_memory_tiered_reservation_score_store();
        let item = new_tiered_reservation_score("code-612", "label-217");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
