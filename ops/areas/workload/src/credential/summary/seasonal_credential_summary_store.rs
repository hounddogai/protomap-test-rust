use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal credential summaries by code.
pub trait SeasonalCredentialSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalCredentialSummary>;
    fn save(&self, item: &SeasonalCredentialSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalCredentialSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalCredentialSummaryStore {
    pub items: RefCell<BTreeMap<String, SeasonalCredentialSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalCredentialSummaryStore for MemorySeasonalCredentialSummaryStore {
    fn find(&self, code: &str) -> Option<SeasonalCredentialSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalCredentialSummary) {
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

    fn list(&self) -> Vec<SeasonalCredentialSummary> {
        let mut out: Vec<SeasonalCredentialSummary> = Vec::new();
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

pub fn new_memory_seasonal_credential_summary_store() -> Rc<dyn SeasonalCredentialSummaryStore> {
    Rc::new(MemorySeasonalCredentialSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalCredentialSummaryStore {
    pub inner: Rc<dyn SeasonalCredentialSummaryStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalCredentialSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalCredentialSummaryStore for CachedSeasonalCredentialSummaryStore {
    fn find(&self, code: &str) -> Option<SeasonalCredentialSummary> {
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

    fn save(&self, item: &SeasonalCredentialSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalCredentialSummary> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_credential_summary_store(
    inner: Rc<dyn SeasonalCredentialSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalCredentialSummaryStore> {
    Rc::new(CachedSeasonalCredentialSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalCredentialSummary>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal credential summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSummaryQuery {
    pub segment: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalCredentialSummaryQuery {
    pub fn matches(&self, item: &SeasonalCredentialSummary) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalCredentialSummary]) -> Vec<SeasonalCredentialSummary> {
        let matched = collections::filter(items, |candidate: &SeasonalCredentialSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_credential_summaries(
    store: Rc<dyn SeasonalCredentialSummaryStore>,
    query: &SeasonalCredentialSummaryQuery,
) -> Vec<SeasonalCredentialSummary> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalCredentialSummaryStore> = new_memory_seasonal_credential_summary_store();
        let item = new_seasonal_credential_summary("code-781", "title-217");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
