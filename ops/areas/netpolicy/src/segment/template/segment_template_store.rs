use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps segment templates by code.
pub trait SegmentTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SegmentTemplate>;
    fn save(&self, item: &SegmentTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SegmentTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySegmentTemplateStore {
    pub items: RefCell<BTreeMap<String, SegmentTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl SegmentTemplateStore for MemorySegmentTemplateStore {
    fn find(&self, code: &str) -> Option<SegmentTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SegmentTemplate) {
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

    fn list(&self) -> Vec<SegmentTemplate> {
        let mut out: Vec<SegmentTemplate> = Vec::new();
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

pub fn new_memory_segment_template_store() -> Rc<dyn SegmentTemplateStore> {
    Rc::new(MemorySegmentTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSegmentTemplateStore {
    pub inner: Rc<dyn SegmentTemplateStore>,
    pub cache: RefCell<cache::TtlCache<SegmentTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SegmentTemplateStore for CachedSegmentTemplateStore {
    fn find(&self, code: &str) -> Option<SegmentTemplate> {
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

    fn save(&self, item: &SegmentTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SegmentTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_segment_template_store(
    inner: Rc<dyn SegmentTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SegmentTemplateStore> {
    Rc::new(CachedSegmentTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SegmentTemplate>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects segment templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentTemplateQuery {
    pub unit: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SegmentTemplateQuery {
    pub fn matches(&self, item: &SegmentTemplate) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SegmentTemplate]) -> Vec<SegmentTemplate> {
        let matched = collections::filter(items, |candidate: &SegmentTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_segment_templates(
    store: Rc<dyn SegmentTemplateStore>,
    query: &SegmentTemplateQuery,
) -> Vec<SegmentTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SegmentTemplateStore> = new_memory_segment_template_store();
        let item = new_segment_template("code-543", "label-127");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
