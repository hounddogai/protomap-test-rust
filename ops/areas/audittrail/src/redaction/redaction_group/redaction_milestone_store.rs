use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps redaction milestones by code.
pub trait RedactionMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RedactionMilestone>;
    fn save(&self, item: &RedactionMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RedactionMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRedactionMilestoneStore {
    pub items: RefCell<BTreeMap<String, RedactionMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl RedactionMilestoneStore for MemoryRedactionMilestoneStore {
    fn find(&self, code: &str) -> Option<RedactionMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RedactionMilestone) {
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

    fn list(&self) -> Vec<RedactionMilestone> {
        let mut out: Vec<RedactionMilestone> = Vec::new();
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

pub fn new_memory_redaction_milestone_store() -> Rc<dyn RedactionMilestoneStore> {
    Rc::new(MemoryRedactionMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRedactionMilestoneStore {
    pub inner: Rc<dyn RedactionMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<RedactionMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RedactionMilestoneStore for CachedRedactionMilestoneStore {
    fn find(&self, code: &str) -> Option<RedactionMilestone> {
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

    fn save(&self, item: &RedactionMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RedactionMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_redaction_milestone_store(
    inner: Rc<dyn RedactionMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RedactionMilestoneStore> {
    Rc::new(CachedRedactionMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RedactionMilestone>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects redaction milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMilestoneQuery {
    pub cluster: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RedactionMilestoneQuery {
    pub fn matches(&self, item: &RedactionMilestone) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[RedactionMilestone]) -> Vec<RedactionMilestone> {
        let matched = collections::filter(items, |candidate: &RedactionMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_redaction_milestones(
    store: Rc<dyn RedactionMilestoneStore>,
    query: &RedactionMilestoneQuery,
) -> Vec<RedactionMilestone> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RedactionMilestoneStore> = new_memory_redaction_milestone_store();
        let item = new_redaction_milestone("code-329", "name-800");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
