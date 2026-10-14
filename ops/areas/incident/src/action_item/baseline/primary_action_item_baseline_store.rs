use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary action item baselines by code.
pub trait PrimaryActionItemBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryActionItemBaseline>;
    fn save(&self, item: &PrimaryActionItemBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryActionItemBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryActionItemBaselineStore {
    pub items: RefCell<BTreeMap<String, PrimaryActionItemBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryActionItemBaselineStore for MemoryPrimaryActionItemBaselineStore {
    fn find(&self, code: &str) -> Option<PrimaryActionItemBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryActionItemBaseline) {
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

    fn list(&self) -> Vec<PrimaryActionItemBaseline> {
        let mut out: Vec<PrimaryActionItemBaseline> = Vec::new();
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

pub fn new_memory_primary_action_item_baseline_store() -> Rc<dyn PrimaryActionItemBaselineStore> {
    Rc::new(MemoryPrimaryActionItemBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryActionItemBaselineStore {
    pub inner: Rc<dyn PrimaryActionItemBaselineStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryActionItemBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryActionItemBaselineStore for CachedPrimaryActionItemBaselineStore {
    fn find(&self, code: &str) -> Option<PrimaryActionItemBaseline> {
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

    fn save(&self, item: &PrimaryActionItemBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryActionItemBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_primary_action_item_baseline_store(
    inner: Rc<dyn PrimaryActionItemBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryActionItemBaselineStore> {
    Rc::new(CachedPrimaryActionItemBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryActionItemBaseline>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary action item baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBaselineQuery {
    pub country: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryActionItemBaselineQuery {
    pub fn matches(&self, item: &PrimaryActionItemBaseline) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryActionItemBaseline]) -> Vec<PrimaryActionItemBaseline> {
        let matched = collections::filter(items, |candidate: &PrimaryActionItemBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_action_item_baselines(
    store: Rc<dyn PrimaryActionItemBaselineStore>,
    query: &PrimaryActionItemBaselineQuery,
) -> Vec<PrimaryActionItemBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryActionItemBaselineStore> = new_memory_primary_action_item_baseline_store();
        let item = new_primary_action_item_baseline("code-933", "title-185");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
