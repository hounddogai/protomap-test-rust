use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps policy templates by code.
pub trait PolicyTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PolicyTemplate>;
    fn save(&self, item: &PolicyTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PolicyTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPolicyTemplateStore {
    pub items: RefCell<BTreeMap<String, PolicyTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl PolicyTemplateStore for MemoryPolicyTemplateStore {
    fn find(&self, code: &str) -> Option<PolicyTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PolicyTemplate) {
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

    fn list(&self) -> Vec<PolicyTemplate> {
        let mut out: Vec<PolicyTemplate> = Vec::new();
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

pub fn new_memory_policy_template_store() -> Rc<dyn PolicyTemplateStore> {
    Rc::new(MemoryPolicyTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPolicyTemplateStore {
    pub inner: Rc<dyn PolicyTemplateStore>,
    pub cache: RefCell<cache::TtlCache<PolicyTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PolicyTemplateStore for CachedPolicyTemplateStore {
    fn find(&self, code: &str) -> Option<PolicyTemplate> {
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

    fn save(&self, item: &PolicyTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PolicyTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_policy_template_store(
    inner: Rc<dyn PolicyTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PolicyTemplateStore> {
    Rc::new(CachedPolicyTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PolicyTemplate>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects policy templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyTemplateQuery {
    pub category: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PolicyTemplateQuery {
    pub fn matches(&self, item: &PolicyTemplate) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[PolicyTemplate]) -> Vec<PolicyTemplate> {
        let matched = collections::filter(items, |candidate: &PolicyTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_policy_templates(store: Rc<dyn PolicyTemplateStore>, query: &PolicyTemplateQuery) -> Vec<PolicyTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PolicyTemplateStore> = new_memory_policy_template_store();
        let item = new_policy_template("code-856", "label-126");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
