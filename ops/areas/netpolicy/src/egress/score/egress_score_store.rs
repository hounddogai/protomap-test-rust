use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps egress scores by code.
pub trait EgressScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EgressScore>;
    fn save(&self, item: &EgressScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EgressScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEgressScoreStore {
    pub items: RefCell<BTreeMap<String, EgressScore>>,
    pub order: RefCell<Vec<String>>,
}

impl EgressScoreStore for MemoryEgressScoreStore {
    fn find(&self, code: &str) -> Option<EgressScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EgressScore) {
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

    fn list(&self) -> Vec<EgressScore> {
        let mut out: Vec<EgressScore> = Vec::new();
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

pub fn new_memory_egress_score_store() -> Rc<dyn EgressScoreStore> {
    Rc::new(MemoryEgressScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEgressScoreStore {
    pub inner: Rc<dyn EgressScoreStore>,
    pub cache: RefCell<cache::TtlCache<EgressScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EgressScoreStore for CachedEgressScoreStore {
    fn find(&self, code: &str) -> Option<EgressScore> {
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

    fn save(&self, item: &EgressScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EgressScore> {
        self.inner.list()
    }
}

pub fn new_cached_egress_score_store(
    inner: Rc<dyn EgressScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EgressScoreStore> {
    Rc::new(CachedEgressScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EgressScore>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects egress scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressScoreQuery {
    pub locale: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EgressScoreQuery {
    pub fn matches(&self, item: &EgressScore) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EgressScore]) -> Vec<EgressScore> {
        let matched = collections::filter(items, |candidate: &EgressScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_egress_scores(store: Rc<dyn EgressScoreStore>, query: &EgressScoreQuery) -> Vec<EgressScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EgressScoreStore> = new_memory_egress_score_store();
        let item = new_egress_score("code-504", "label-313");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
