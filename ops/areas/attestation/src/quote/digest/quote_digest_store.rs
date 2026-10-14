use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps quote digests by code.
pub trait QuoteDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QuoteDigest>;
    fn save(&self, item: &QuoteDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QuoteDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQuoteDigestStore {
    pub items: RefCell<BTreeMap<String, QuoteDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl QuoteDigestStore for MemoryQuoteDigestStore {
    fn find(&self, code: &str) -> Option<QuoteDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QuoteDigest) {
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

    fn list(&self) -> Vec<QuoteDigest> {
        let mut out: Vec<QuoteDigest> = Vec::new();
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

pub fn new_memory_quote_digest_store() -> Rc<dyn QuoteDigestStore> {
    Rc::new(MemoryQuoteDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQuoteDigestStore {
    pub inner: Rc<dyn QuoteDigestStore>,
    pub cache: RefCell<cache::TtlCache<QuoteDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QuoteDigestStore for CachedQuoteDigestStore {
    fn find(&self, code: &str) -> Option<QuoteDigest> {
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

    fn save(&self, item: &QuoteDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QuoteDigest> {
        self.inner.list()
    }
}

pub fn new_cached_quote_digest_store(
    inner: Rc<dyn QuoteDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QuoteDigestStore> {
    Rc::new(CachedQuoteDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QuoteDigest>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects quote digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteDigestQuery {
    pub unit: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QuoteDigestQuery {
    pub fn matches(&self, item: &QuoteDigest) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QuoteDigest]) -> Vec<QuoteDigest> {
        let matched = collections::filter(items, |candidate: &QuoteDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_quote_digests(store: Rc<dyn QuoteDigestStore>, query: &QuoteDigestQuery) -> Vec<QuoteDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn QuoteDigestStore> = new_memory_quote_digest_store();
        let item = new_quote_digest("code-720", "name-195");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
