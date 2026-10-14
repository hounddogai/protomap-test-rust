use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict quote baselines by code.
pub trait StrictQuoteBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictQuoteBaseline>;
    fn save(&self, item: &StrictQuoteBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictQuoteBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictQuoteBaselineStore {
    pub items: RefCell<BTreeMap<String, StrictQuoteBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictQuoteBaselineStore for MemoryStrictQuoteBaselineStore {
    fn find(&self, code: &str) -> Option<StrictQuoteBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictQuoteBaseline) {
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

    fn list(&self) -> Vec<StrictQuoteBaseline> {
        let mut out: Vec<StrictQuoteBaseline> = Vec::new();
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

pub fn new_memory_strict_quote_baseline_store() -> Rc<dyn StrictQuoteBaselineStore> {
    Rc::new(MemoryStrictQuoteBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictQuoteBaselineStore {
    pub inner: Rc<dyn StrictQuoteBaselineStore>,
    pub cache: RefCell<cache::TtlCache<StrictQuoteBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictQuoteBaselineStore for CachedStrictQuoteBaselineStore {
    fn find(&self, code: &str) -> Option<StrictQuoteBaseline> {
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

    fn save(&self, item: &StrictQuoteBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictQuoteBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_strict_quote_baseline_store(
    inner: Rc<dyn StrictQuoteBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictQuoteBaselineStore> {
    Rc::new(CachedStrictQuoteBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictQuoteBaseline>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict quote baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictQuoteBaselineQuery {
    pub title: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictQuoteBaselineQuery {
    pub fn matches(&self, item: &StrictQuoteBaseline) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictQuoteBaseline]) -> Vec<StrictQuoteBaseline> {
        let matched = collections::filter(items, |candidate: &StrictQuoteBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_quote_baselines(
    store: Rc<dyn StrictQuoteBaselineStore>,
    query: &StrictQuoteBaselineQuery,
) -> Vec<StrictQuoteBaseline> {
    query.apply(&store.list())
}
