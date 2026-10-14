use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps quote tiers by code.
pub trait QuoteTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QuoteTier>;
    fn save(&self, item: &QuoteTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QuoteTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQuoteTierStore {
    pub items: RefCell<BTreeMap<String, QuoteTier>>,
    pub order: RefCell<Vec<String>>,
}

impl QuoteTierStore for MemoryQuoteTierStore {
    fn find(&self, code: &str) -> Option<QuoteTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QuoteTier) {
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

    fn list(&self) -> Vec<QuoteTier> {
        let mut out: Vec<QuoteTier> = Vec::new();
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

pub fn new_memory_quote_tier_store() -> Rc<dyn QuoteTierStore> {
    Rc::new(MemoryQuoteTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQuoteTierStore {
    pub inner: Rc<dyn QuoteTierStore>,
    pub cache: RefCell<cache::TtlCache<QuoteTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QuoteTierStore for CachedQuoteTierStore {
    fn find(&self, code: &str) -> Option<QuoteTier> {
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

    fn save(&self, item: &QuoteTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QuoteTier> {
        self.inner.list()
    }
}

pub fn new_cached_quote_tier_store(
    inner: Rc<dyn QuoteTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QuoteTierStore> {
    Rc::new(CachedQuoteTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QuoteTier>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects quote tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTierQuery {
    pub locale: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QuoteTierQuery {
    pub fn matches(&self, item: &QuoteTier) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QuoteTier]) -> Vec<QuoteTier> {
        let matched = collections::filter(items, |candidate: &QuoteTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_quote_tiers(store: Rc<dyn QuoteTierStore>, query: &QuoteTierQuery) -> Vec<QuoteTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn QuoteTierStore> = new_memory_quote_tier_store();
        let item = new_quote_tier("code-72", "title-822");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
