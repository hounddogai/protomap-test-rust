use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps quote targets by code.
pub trait QuoteTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QuoteTarget>;
    fn save(&self, item: &QuoteTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QuoteTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQuoteTargetStore {
    pub items: RefCell<BTreeMap<String, QuoteTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl QuoteTargetStore for MemoryQuoteTargetStore {
    fn find(&self, code: &str) -> Option<QuoteTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QuoteTarget) {
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

    fn list(&self) -> Vec<QuoteTarget> {
        let mut out: Vec<QuoteTarget> = Vec::new();
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

pub fn new_memory_quote_target_store() -> Rc<dyn QuoteTargetStore> {
    Rc::new(MemoryQuoteTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQuoteTargetStore {
    pub inner: Rc<dyn QuoteTargetStore>,
    pub cache: RefCell<cache::TtlCache<QuoteTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QuoteTargetStore for CachedQuoteTargetStore {
    fn find(&self, code: &str) -> Option<QuoteTarget> {
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

    fn save(&self, item: &QuoteTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QuoteTarget> {
        self.inner.list()
    }
}

pub fn new_cached_quote_target_store(
    inner: Rc<dyn QuoteTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QuoteTargetStore> {
    Rc::new(CachedQuoteTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QuoteTarget>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects quote targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTargetQuery {
    pub category: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QuoteTargetQuery {
    pub fn matches(&self, item: &QuoteTarget) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QuoteTarget]) -> Vec<QuoteTarget> {
        let matched = collections::filter(items, |candidate: &QuoteTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_quote_targets(store: Rc<dyn QuoteTargetStore>, query: &QuoteTargetQuery) -> Vec<QuoteTarget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn QuoteTargetStore> = new_memory_quote_target_store();
        let item = new_quote_target("code-797", "name-925");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
