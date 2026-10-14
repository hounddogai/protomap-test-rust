use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps action item preferences by code.
pub trait ActionItemPreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ActionItemPreference>;
    fn save(&self, item: &ActionItemPreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ActionItemPreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryActionItemPreferenceStore {
    pub items: RefCell<BTreeMap<String, ActionItemPreference>>,
    pub order: RefCell<Vec<String>>,
}

impl ActionItemPreferenceStore for MemoryActionItemPreferenceStore {
    fn find(&self, code: &str) -> Option<ActionItemPreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ActionItemPreference) {
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

    fn list(&self) -> Vec<ActionItemPreference> {
        let mut out: Vec<ActionItemPreference> = Vec::new();
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

pub fn new_memory_action_item_preference_store() -> Rc<dyn ActionItemPreferenceStore> {
    Rc::new(MemoryActionItemPreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedActionItemPreferenceStore {
    pub inner: Rc<dyn ActionItemPreferenceStore>,
    pub cache: RefCell<cache::TtlCache<ActionItemPreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ActionItemPreferenceStore for CachedActionItemPreferenceStore {
    fn find(&self, code: &str) -> Option<ActionItemPreference> {
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

    fn save(&self, item: &ActionItemPreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ActionItemPreference> {
        self.inner.list()
    }
}

pub fn new_cached_action_item_preference_store(
    inner: Rc<dyn ActionItemPreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ActionItemPreferenceStore> {
    Rc::new(CachedActionItemPreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ActionItemPreference>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects action item preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPreferenceQuery {
    pub channel: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ActionItemPreferenceQuery {
    pub fn matches(&self, item: &ActionItemPreference) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ActionItemPreference]) -> Vec<ActionItemPreference> {
        let matched = collections::filter(items, |candidate: &ActionItemPreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_action_item_preferences(
    store: Rc<dyn ActionItemPreferenceStore>,
    query: &ActionItemPreferenceQuery,
) -> Vec<ActionItemPreference> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ActionItemPreferenceStore> = new_memory_action_item_preference_store();
        let item = new_action_item_preference("code-572", "title-999");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
