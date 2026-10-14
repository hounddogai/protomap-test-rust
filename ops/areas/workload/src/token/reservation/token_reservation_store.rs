use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps token reservations by code.
pub trait TokenReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TokenReservation>;
    fn save(&self, item: &TokenReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TokenReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTokenReservationStore {
    pub items: RefCell<BTreeMap<String, TokenReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl TokenReservationStore for MemoryTokenReservationStore {
    fn find(&self, code: &str) -> Option<TokenReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TokenReservation) {
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

    fn list(&self) -> Vec<TokenReservation> {
        let mut out: Vec<TokenReservation> = Vec::new();
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

pub fn new_memory_token_reservation_store() -> Rc<dyn TokenReservationStore> {
    Rc::new(MemoryTokenReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTokenReservationStore {
    pub inner: Rc<dyn TokenReservationStore>,
    pub cache: RefCell<cache::TtlCache<TokenReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TokenReservationStore for CachedTokenReservationStore {
    fn find(&self, code: &str) -> Option<TokenReservation> {
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

    fn save(&self, item: &TokenReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TokenReservation> {
        self.inner.list()
    }
}

pub fn new_cached_token_reservation_store(
    inner: Rc<dyn TokenReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TokenReservationStore> {
    Rc::new(CachedTokenReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TokenReservation>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects token reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenReservationQuery {
    pub scope: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TokenReservationQuery {
    pub fn matches(&self, item: &TokenReservation) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TokenReservation]) -> Vec<TokenReservation> {
        let matched = collections::filter(items, |candidate: &TokenReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_token_reservations(
    store: Rc<dyn TokenReservationStore>,
    query: &TokenReservationQuery,
) -> Vec<TokenReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TokenReservationStore> = new_memory_token_reservation_store();
        let item = new_token_reservation("code-369", "title-702");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
