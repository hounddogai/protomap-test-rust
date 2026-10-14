use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps silence adjustments by code.
pub trait SilenceAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SilenceAdjustment>;
    fn save(&self, item: &SilenceAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SilenceAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySilenceAdjustmentStore {
    pub items: RefCell<BTreeMap<String, SilenceAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl SilenceAdjustmentStore for MemorySilenceAdjustmentStore {
    fn find(&self, code: &str) -> Option<SilenceAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SilenceAdjustment) {
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

    fn list(&self) -> Vec<SilenceAdjustment> {
        let mut out: Vec<SilenceAdjustment> = Vec::new();
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

pub fn new_memory_silence_adjustment_store() -> Rc<dyn SilenceAdjustmentStore> {
    Rc::new(MemorySilenceAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSilenceAdjustmentStore {
    pub inner: Rc<dyn SilenceAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<SilenceAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SilenceAdjustmentStore for CachedSilenceAdjustmentStore {
    fn find(&self, code: &str) -> Option<SilenceAdjustment> {
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

    fn save(&self, item: &SilenceAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SilenceAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_silence_adjustment_store(
    inner: Rc<dyn SilenceAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SilenceAdjustmentStore> {
    Rc::new(CachedSilenceAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SilenceAdjustment>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects silence adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAdjustmentQuery {
    pub region: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SilenceAdjustmentQuery {
    pub fn matches(&self, item: &SilenceAdjustment) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[SilenceAdjustment]) -> Vec<SilenceAdjustment> {
        let matched = collections::filter(items, |candidate: &SilenceAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_silence_adjustments(
    store: Rc<dyn SilenceAdjustmentStore>,
    query: &SilenceAdjustmentQuery,
) -> Vec<SilenceAdjustment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SilenceAdjustmentStore> = new_memory_silence_adjustment_store();
        let item = new_silence_adjustment("code-630", "label-130");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
