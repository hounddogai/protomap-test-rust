use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal migration scores by code.
pub trait SeasonalMigrationScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalMigrationScore>;
    fn save(&self, item: &SeasonalMigrationScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalMigrationScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalMigrationScoreStore {
    pub items: RefCell<BTreeMap<String, SeasonalMigrationScore>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalMigrationScoreStore for MemorySeasonalMigrationScoreStore {
    fn find(&self, code: &str) -> Option<SeasonalMigrationScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalMigrationScore) {
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

    fn list(&self) -> Vec<SeasonalMigrationScore> {
        let mut out: Vec<SeasonalMigrationScore> = Vec::new();
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

pub fn new_memory_seasonal_migration_score_store() -> Rc<dyn SeasonalMigrationScoreStore> {
    Rc::new(MemorySeasonalMigrationScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalMigrationScoreStore {
    pub inner: Rc<dyn SeasonalMigrationScoreStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalMigrationScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalMigrationScoreStore for CachedSeasonalMigrationScoreStore {
    fn find(&self, code: &str) -> Option<SeasonalMigrationScore> {
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

    fn save(&self, item: &SeasonalMigrationScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalMigrationScore> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_migration_score_store(
    inner: Rc<dyn SeasonalMigrationScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalMigrationScoreStore> {
    Rc::new(CachedSeasonalMigrationScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalMigrationScore>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal migration scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMigrationScoreQuery {
    pub title: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalMigrationScoreQuery {
    pub fn matches(&self, item: &SeasonalMigrationScore) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalMigrationScore]) -> Vec<SeasonalMigrationScore> {
        let matched = collections::filter(items, |candidate: &SeasonalMigrationScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_migration_scores(
    store: Rc<dyn SeasonalMigrationScoreStore>,
    query: &SeasonalMigrationScoreQuery,
) -> Vec<SeasonalMigrationScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalMigrationScoreStore> = new_memory_seasonal_migration_score_store();
        let item = new_seasonal_migration_score("code-504", "title-673");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
