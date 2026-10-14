use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional usage scores by code.
pub trait RegionalUsageScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalUsageScore>;
    fn save(&self, item: &RegionalUsageScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalUsageScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalUsageScoreStore {
    pub items: RefCell<BTreeMap<String, RegionalUsageScore>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalUsageScoreStore for MemoryRegionalUsageScoreStore {
    fn find(&self, code: &str) -> Option<RegionalUsageScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalUsageScore) {
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

    fn list(&self) -> Vec<RegionalUsageScore> {
        let mut out: Vec<RegionalUsageScore> = Vec::new();
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

pub fn new_memory_regional_usage_score_store() -> Rc<dyn RegionalUsageScoreStore> {
    Rc::new(MemoryRegionalUsageScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalUsageScoreStore {
    pub inner: Rc<dyn RegionalUsageScoreStore>,
    pub cache: RefCell<cache::TtlCache<RegionalUsageScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalUsageScoreStore for CachedRegionalUsageScoreStore {
    fn find(&self, code: &str) -> Option<RegionalUsageScore> {
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

    fn save(&self, item: &RegionalUsageScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalUsageScore> {
        self.inner.list()
    }
}

pub fn new_cached_regional_usage_score_store(
    inner: Rc<dyn RegionalUsageScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalUsageScoreStore> {
    Rc::new(CachedRegionalUsageScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalUsageScore>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional usage scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageScoreQuery {
    pub segment: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalUsageScoreQuery {
    pub fn matches(&self, item: &RegionalUsageScore) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalUsageScore]) -> Vec<RegionalUsageScore> {
        let matched = collections::filter(items, |candidate: &RegionalUsageScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_usage_scores(
    store: Rc<dyn RegionalUsageScoreStore>,
    query: &RegionalUsageScoreQuery,
) -> Vec<RegionalUsageScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalUsageScoreStore> = new_memory_regional_usage_score_store();
        let item = new_regional_usage_score("code-298", "title-335");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
