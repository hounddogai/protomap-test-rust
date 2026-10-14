use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps flow log windows by code.
pub trait FlowLogWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FlowLogWindow>;
    fn save(&self, item: &FlowLogWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FlowLogWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFlowLogWindowStore {
    pub items: RefCell<BTreeMap<String, FlowLogWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl FlowLogWindowStore for MemoryFlowLogWindowStore {
    fn find(&self, code: &str) -> Option<FlowLogWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FlowLogWindow) {
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

    fn list(&self) -> Vec<FlowLogWindow> {
        let mut out: Vec<FlowLogWindow> = Vec::new();
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

pub fn new_memory_flow_log_window_store() -> Rc<dyn FlowLogWindowStore> {
    Rc::new(MemoryFlowLogWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFlowLogWindowStore {
    pub inner: Rc<dyn FlowLogWindowStore>,
    pub cache: RefCell<cache::TtlCache<FlowLogWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FlowLogWindowStore for CachedFlowLogWindowStore {
    fn find(&self, code: &str) -> Option<FlowLogWindow> {
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

    fn save(&self, item: &FlowLogWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FlowLogWindow> {
        self.inner.list()
    }
}

pub fn new_cached_flow_log_window_store(
    inner: Rc<dyn FlowLogWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FlowLogWindowStore> {
    Rc::new(CachedFlowLogWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FlowLogWindow>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects flow log windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogWindowQuery {
    pub name: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FlowLogWindowQuery {
    pub fn matches(&self, item: &FlowLogWindow) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FlowLogWindow]) -> Vec<FlowLogWindow> {
        let matched = collections::filter(items, |candidate: &FlowLogWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_flow_log_windows(store: Rc<dyn FlowLogWindowStore>, query: &FlowLogWindowQuery) -> Vec<FlowLogWindow> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FlowLogWindowStore> = new_memory_flow_log_window_store();
        let item = new_flow_log_window("code-335", "name-538");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
