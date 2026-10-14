use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps peering scores by code.
pub trait PeeringScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PeeringScore>;
    fn save(&self, item: &PeeringScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PeeringScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPeeringScoreStore {
    pub items: RefCell<BTreeMap<String, PeeringScore>>,
    pub order: RefCell<Vec<String>>,
}

impl PeeringScoreStore for MemoryPeeringScoreStore {
    fn find(&self, code: &str) -> Option<PeeringScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PeeringScore) {
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

    fn list(&self) -> Vec<PeeringScore> {
        let mut out: Vec<PeeringScore> = Vec::new();
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

pub fn new_memory_peering_score_store() -> Rc<dyn PeeringScoreStore> {
    Rc::new(MemoryPeeringScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPeeringScoreStore {
    pub inner: Rc<dyn PeeringScoreStore>,
    pub cache: RefCell<cache::TtlCache<PeeringScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PeeringScoreStore for CachedPeeringScoreStore {
    fn find(&self, code: &str) -> Option<PeeringScore> {
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

    fn save(&self, item: &PeeringScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PeeringScore> {
        self.inner.list()
    }
}

pub fn new_cached_peering_score_store(
    inner: Rc<dyn PeeringScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PeeringScoreStore> {
    Rc::new(CachedPeeringScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PeeringScore>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects peering scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringScoreQuery {
    pub label: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PeeringScoreQuery {
    pub fn matches(&self, item: &PeeringScore) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[PeeringScore]) -> Vec<PeeringScore> {
        let matched = collections::filter(items, |candidate: &PeeringScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_peering_scores(store: Rc<dyn PeeringScoreStore>, query: &PeeringScoreQuery) -> Vec<PeeringScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PeeringScoreStore> = new_memory_peering_score_store();
        let item = new_peering_score("code-536", "label-899");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
