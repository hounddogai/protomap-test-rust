use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient freeze tiers by code.
pub trait LenientFreezeTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientFreezeTier>;
    fn save(&self, item: &LenientFreezeTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientFreezeTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientFreezeTierStore {
    pub items: RefCell<BTreeMap<String, LenientFreezeTier>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientFreezeTierStore for MemoryLenientFreezeTierStore {
    fn find(&self, code: &str) -> Option<LenientFreezeTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientFreezeTier) {
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

    fn list(&self) -> Vec<LenientFreezeTier> {
        let mut out: Vec<LenientFreezeTier> = Vec::new();
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

pub fn new_memory_lenient_freeze_tier_store() -> Rc<dyn LenientFreezeTierStore> {
    Rc::new(MemoryLenientFreezeTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientFreezeTierStore {
    pub inner: Rc<dyn LenientFreezeTierStore>,
    pub cache: RefCell<cache::TtlCache<LenientFreezeTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientFreezeTierStore for CachedLenientFreezeTierStore {
    fn find(&self, code: &str) -> Option<LenientFreezeTier> {
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

    fn save(&self, item: &LenientFreezeTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientFreezeTier> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_freeze_tier_store(
    inner: Rc<dyn LenientFreezeTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientFreezeTierStore> {
    Rc::new(CachedLenientFreezeTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientFreezeTier>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient freeze tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeTierQuery {
    pub namespace: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientFreezeTierQuery {
    pub fn matches(&self, item: &LenientFreezeTier) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientFreezeTier]) -> Vec<LenientFreezeTier> {
        let matched = collections::filter(items, |candidate: &LenientFreezeTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_freeze_tiers(
    store: Rc<dyn LenientFreezeTierStore>,
    query: &LenientFreezeTierQuery,
) -> Vec<LenientFreezeTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientFreezeTierStore> = new_memory_lenient_freeze_tier_store();
        let item = new_lenient_freeze_tier("code-122", "title-976");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
