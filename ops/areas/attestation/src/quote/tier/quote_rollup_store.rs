use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps quote rollups by code.
pub trait QuoteRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QuoteRollup>;
    fn save(&self, item: &QuoteRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QuoteRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQuoteRollupStore {
    pub items: RefCell<BTreeMap<String, QuoteRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl QuoteRollupStore for MemoryQuoteRollupStore {
    fn find(&self, code: &str) -> Option<QuoteRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QuoteRollup) {
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

    fn list(&self) -> Vec<QuoteRollup> {
        let mut out: Vec<QuoteRollup> = Vec::new();
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

pub fn new_memory_quote_rollup_store() -> Rc<dyn QuoteRollupStore> {
    Rc::new(MemoryQuoteRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQuoteRollupStore {
    pub inner: Rc<dyn QuoteRollupStore>,
    pub cache: RefCell<cache::TtlCache<QuoteRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QuoteRollupStore for CachedQuoteRollupStore {
    fn find(&self, code: &str) -> Option<QuoteRollup> {
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

    fn save(&self, item: &QuoteRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QuoteRollup> {
        self.inner.list()
    }
}

pub fn new_cached_quote_rollup_store(
    inner: Rc<dyn QuoteRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QuoteRollupStore> {
    Rc::new(CachedQuoteRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QuoteRollup>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects quote rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRollupQuery {
    pub label: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QuoteRollupQuery {
    pub fn matches(&self, item: &QuoteRollup) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[QuoteRollup]) -> Vec<QuoteRollup> {
        let matched = collections::filter(items, |candidate: &QuoteRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_quote_rollups(store: Rc<dyn QuoteRollupStore>, query: &QuoteRollupQuery) -> Vec<QuoteRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn QuoteRollupStore> = new_memory_quote_rollup_store();
        let item = new_quote_rollup("code-657", "label-898");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
