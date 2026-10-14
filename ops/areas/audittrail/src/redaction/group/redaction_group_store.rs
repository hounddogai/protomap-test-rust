use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps redaction groups by code.
pub trait RedactionGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RedactionGroup>;
    fn save(&self, item: &RedactionGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RedactionGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRedactionGroupStore {
    pub items: RefCell<BTreeMap<String, RedactionGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl RedactionGroupStore for MemoryRedactionGroupStore {
    fn find(&self, code: &str) -> Option<RedactionGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RedactionGroup) {
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

    fn list(&self) -> Vec<RedactionGroup> {
        let mut out: Vec<RedactionGroup> = Vec::new();
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

pub fn new_memory_redaction_group_store() -> Rc<dyn RedactionGroupStore> {
    Rc::new(MemoryRedactionGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRedactionGroupStore {
    pub inner: Rc<dyn RedactionGroupStore>,
    pub cache: RefCell<cache::TtlCache<RedactionGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RedactionGroupStore for CachedRedactionGroupStore {
    fn find(&self, code: &str) -> Option<RedactionGroup> {
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

    fn save(&self, item: &RedactionGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RedactionGroup> {
        self.inner.list()
    }
}

pub fn new_cached_redaction_group_store(
    inner: Rc<dyn RedactionGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RedactionGroupStore> {
    Rc::new(CachedRedactionGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RedactionGroup>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects redaction groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionGroupQuery {
    pub channel: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RedactionGroupQuery {
    pub fn matches(&self, item: &RedactionGroup) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RedactionGroup]) -> Vec<RedactionGroup> {
        let matched = collections::filter(items, |candidate: &RedactionGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_redaction_groups(store: Rc<dyn RedactionGroupStore>, query: &RedactionGroupQuery) -> Vec<RedactionGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RedactionGroupStore> = new_memory_redaction_group_store();
        let item = new_redaction_group("code-229", "label-942");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
