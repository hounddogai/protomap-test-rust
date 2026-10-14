use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional redaction groups by code.
pub trait RegionalRedactionGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalRedactionGroup>;
    fn save(&self, item: &RegionalRedactionGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalRedactionGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalRedactionGroupStore {
    pub items: RefCell<BTreeMap<String, RegionalRedactionGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalRedactionGroupStore for MemoryRegionalRedactionGroupStore {
    fn find(&self, code: &str) -> Option<RegionalRedactionGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalRedactionGroup) {
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

    fn list(&self) -> Vec<RegionalRedactionGroup> {
        let mut out: Vec<RegionalRedactionGroup> = Vec::new();
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

pub fn new_memory_regional_redaction_group_store() -> Rc<dyn RegionalRedactionGroupStore> {
    Rc::new(MemoryRegionalRedactionGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalRedactionGroupStore {
    pub inner: Rc<dyn RegionalRedactionGroupStore>,
    pub cache: RefCell<cache::TtlCache<RegionalRedactionGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalRedactionGroupStore for CachedRegionalRedactionGroupStore {
    fn find(&self, code: &str) -> Option<RegionalRedactionGroup> {
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

    fn save(&self, item: &RegionalRedactionGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalRedactionGroup> {
        self.inner.list()
    }
}

pub fn new_cached_regional_redaction_group_store(
    inner: Rc<dyn RegionalRedactionGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalRedactionGroupStore> {
    Rc::new(CachedRegionalRedactionGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalRedactionGroup>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional redaction groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionGroupQuery {
    pub segment: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalRedactionGroupQuery {
    pub fn matches(&self, item: &RegionalRedactionGroup) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalRedactionGroup]) -> Vec<RegionalRedactionGroup> {
        let matched = collections::filter(items, |candidate: &RegionalRedactionGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_redaction_groups(
    store: Rc<dyn RegionalRedactionGroupStore>,
    query: &RegionalRedactionGroupQuery,
) -> Vec<RegionalRedactionGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalRedactionGroupStore> = new_memory_regional_redaction_group_store();
        let item = new_regional_redaction_group("code-379", "label-322");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
