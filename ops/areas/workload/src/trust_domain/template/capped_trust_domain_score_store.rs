use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped trust domain scores by code.
pub trait CappedTrustDomainScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedTrustDomainScore>;
    fn save(&self, item: &CappedTrustDomainScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedTrustDomainScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedTrustDomainScoreStore {
    pub items: RefCell<BTreeMap<String, CappedTrustDomainScore>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedTrustDomainScoreStore for MemoryCappedTrustDomainScoreStore {
    fn find(&self, code: &str) -> Option<CappedTrustDomainScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedTrustDomainScore) {
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

    fn list(&self) -> Vec<CappedTrustDomainScore> {
        let mut out: Vec<CappedTrustDomainScore> = Vec::new();
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

pub fn new_memory_capped_trust_domain_score_store() -> Rc<dyn CappedTrustDomainScoreStore> {
    Rc::new(MemoryCappedTrustDomainScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedTrustDomainScoreStore {
    pub inner: Rc<dyn CappedTrustDomainScoreStore>,
    pub cache: RefCell<cache::TtlCache<CappedTrustDomainScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedTrustDomainScoreStore for CachedCappedTrustDomainScoreStore {
    fn find(&self, code: &str) -> Option<CappedTrustDomainScore> {
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

    fn save(&self, item: &CappedTrustDomainScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedTrustDomainScore> {
        self.inner.list()
    }
}

pub fn new_cached_capped_trust_domain_score_store(
    inner: Rc<dyn CappedTrustDomainScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedTrustDomainScoreStore> {
    Rc::new(CachedCappedTrustDomainScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedTrustDomainScore>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped trust domain scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainScoreQuery {
    pub label: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedTrustDomainScoreQuery {
    pub fn matches(&self, item: &CappedTrustDomainScore) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[CappedTrustDomainScore]) -> Vec<CappedTrustDomainScore> {
        let matched = collections::filter(items, |candidate: &CappedTrustDomainScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_trust_domain_scores(
    store: Rc<dyn CappedTrustDomainScoreStore>,
    query: &CappedTrustDomainScoreQuery,
) -> Vec<CappedTrustDomainScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedTrustDomainScoreStore> = new_memory_capped_trust_domain_score_store();
        let item = new_capped_trust_domain_score("code-520", "label-89");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
