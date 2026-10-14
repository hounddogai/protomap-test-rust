use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps probe scores by code.
pub trait ProbeScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ProbeScore>;
    fn save(&self, item: &ProbeScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ProbeScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryProbeScoreStore {
    pub items: RefCell<BTreeMap<String, ProbeScore>>,
    pub order: RefCell<Vec<String>>,
}

impl ProbeScoreStore for MemoryProbeScoreStore {
    fn find(&self, code: &str) -> Option<ProbeScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ProbeScore) {
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

    fn list(&self) -> Vec<ProbeScore> {
        let mut out: Vec<ProbeScore> = Vec::new();
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

pub fn new_memory_probe_score_store() -> Rc<dyn ProbeScoreStore> {
    Rc::new(MemoryProbeScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedProbeScoreStore {
    pub inner: Rc<dyn ProbeScoreStore>,
    pub cache: RefCell<cache::TtlCache<ProbeScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ProbeScoreStore for CachedProbeScoreStore {
    fn find(&self, code: &str) -> Option<ProbeScore> {
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

    fn save(&self, item: &ProbeScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ProbeScore> {
        self.inner.list()
    }
}

pub fn new_cached_probe_score_store(
    inner: Rc<dyn ProbeScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ProbeScoreStore> {
    Rc::new(CachedProbeScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ProbeScore>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects probe scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeScoreQuery {
    pub title: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ProbeScoreQuery {
    pub fn matches(&self, item: &ProbeScore) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ProbeScore]) -> Vec<ProbeScore> {
        let matched = collections::filter(items, |candidate: &ProbeScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_probe_scores(store: Rc<dyn ProbeScoreStore>, query: &ProbeScoreQuery) -> Vec<ProbeScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ProbeScoreStore> = new_memory_probe_score_store();
        let item = new_probe_score("code-730", "title-968");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
