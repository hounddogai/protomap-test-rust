use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched policy templates by code.
pub trait BatchedPolicyTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedPolicyTemplate>;
    fn save(&self, item: &BatchedPolicyTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedPolicyTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedPolicyTemplateStore {
    pub items: RefCell<BTreeMap<String, BatchedPolicyTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedPolicyTemplateStore for MemoryBatchedPolicyTemplateStore {
    fn find(&self, code: &str) -> Option<BatchedPolicyTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedPolicyTemplate) {
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

    fn list(&self) -> Vec<BatchedPolicyTemplate> {
        let mut out: Vec<BatchedPolicyTemplate> = Vec::new();
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

pub fn new_memory_batched_policy_template_store() -> Rc<dyn BatchedPolicyTemplateStore> {
    Rc::new(MemoryBatchedPolicyTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedPolicyTemplateStore {
    pub inner: Rc<dyn BatchedPolicyTemplateStore>,
    pub cache: RefCell<cache::TtlCache<BatchedPolicyTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedPolicyTemplateStore for CachedBatchedPolicyTemplateStore {
    fn find(&self, code: &str) -> Option<BatchedPolicyTemplate> {
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

    fn save(&self, item: &BatchedPolicyTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedPolicyTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_batched_policy_template_store(
    inner: Rc<dyn BatchedPolicyTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedPolicyTemplateStore> {
    Rc::new(CachedBatchedPolicyTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedPolicyTemplate>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched policy templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTemplateQuery {
    pub label: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedPolicyTemplateQuery {
    pub fn matches(&self, item: &BatchedPolicyTemplate) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[BatchedPolicyTemplate]) -> Vec<BatchedPolicyTemplate> {
        let matched = collections::filter(items, |candidate: &BatchedPolicyTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_policy_templates(
    store: Rc<dyn BatchedPolicyTemplateStore>,
    query: &BatchedPolicyTemplateQuery,
) -> Vec<BatchedPolicyTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedPolicyTemplateStore> = new_memory_batched_policy_template_store();
        let item = new_batched_policy_template("code-80", "label-452");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
