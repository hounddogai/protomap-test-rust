use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval templates by code.
pub trait ApprovalTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalTemplate>;
    fn save(&self, item: &ApprovalTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalTemplateStore {
    pub items: RefCell<BTreeMap<String, ApprovalTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalTemplateStore for MemoryApprovalTemplateStore {
    fn find(&self, code: &str) -> Option<ApprovalTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalTemplate) {
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

    fn list(&self) -> Vec<ApprovalTemplate> {
        let mut out: Vec<ApprovalTemplate> = Vec::new();
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

pub fn new_memory_approval_template_store() -> Rc<dyn ApprovalTemplateStore> {
    Rc::new(MemoryApprovalTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalTemplateStore {
    pub inner: Rc<dyn ApprovalTemplateStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalTemplateStore for CachedApprovalTemplateStore {
    fn find(&self, code: &str) -> Option<ApprovalTemplate> {
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

    fn save(&self, item: &ApprovalTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_approval_template_store(
    inner: Rc<dyn ApprovalTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalTemplateStore> {
    Rc::new(CachedApprovalTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalTemplate>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalTemplateQuery {
    pub label: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalTemplateQuery {
    pub fn matches(&self, item: &ApprovalTemplate) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[ApprovalTemplate]) -> Vec<ApprovalTemplate> {
        let matched = collections::filter(items, |candidate: &ApprovalTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_templates(
    store: Rc<dyn ApprovalTemplateStore>,
    query: &ApprovalTemplateQuery,
) -> Vec<ApprovalTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ApprovalTemplateStore> = new_memory_approval_template_store();
        let item = new_approval_template("code-878", "label-302");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
