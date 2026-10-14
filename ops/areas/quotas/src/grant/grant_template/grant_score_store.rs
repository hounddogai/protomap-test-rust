use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps grant scores by code.
pub trait GrantScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GrantScore>;
    fn save(&self, item: &GrantScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GrantScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGrantScoreStore {
    pub items: RefCell<BTreeMap<String, GrantScore>>,
    pub order: RefCell<Vec<String>>,
}

impl GrantScoreStore for MemoryGrantScoreStore {
    fn find(&self, code: &str) -> Option<GrantScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GrantScore) {
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

    fn list(&self) -> Vec<GrantScore> {
        let mut out: Vec<GrantScore> = Vec::new();
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

pub fn new_memory_grant_score_store() -> Rc<dyn GrantScoreStore> {
    Rc::new(MemoryGrantScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGrantScoreStore {
    pub inner: Rc<dyn GrantScoreStore>,
    pub cache: RefCell<cache::TtlCache<GrantScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GrantScoreStore for CachedGrantScoreStore {
    fn find(&self, code: &str) -> Option<GrantScore> {
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

    fn save(&self, item: &GrantScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GrantScore> {
        self.inner.list()
    }
}

pub fn new_cached_grant_score_store(
    inner: Rc<dyn GrantScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GrantScoreStore> {
    Rc::new(CachedGrantScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GrantScore>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects grant scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantScoreQuery {
    pub locale: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GrantScoreQuery {
    pub fn matches(&self, item: &GrantScore) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
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

    pub fn apply(&self, items: &[GrantScore]) -> Vec<GrantScore> {
        let matched = collections::filter(items, |candidate: &GrantScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_grant_scores(store: Rc<dyn GrantScoreStore>, query: &GrantScoreQuery) -> Vec<GrantScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GrantScoreStore> = new_memory_grant_score_store();
        let item = new_grant_score("code-699", "name-536");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
