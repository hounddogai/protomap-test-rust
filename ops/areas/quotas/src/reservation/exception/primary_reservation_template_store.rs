use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary reservation templates by code.
pub trait PrimaryReservationTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryReservationTemplate>;
    fn save(&self, item: &PrimaryReservationTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryReservationTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryReservationTemplateStore {
    pub items: RefCell<BTreeMap<String, PrimaryReservationTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryReservationTemplateStore for MemoryPrimaryReservationTemplateStore {
    fn find(&self, code: &str) -> Option<PrimaryReservationTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryReservationTemplate) {
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

    fn list(&self) -> Vec<PrimaryReservationTemplate> {
        let mut out: Vec<PrimaryReservationTemplate> = Vec::new();
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

pub fn new_memory_primary_reservation_template_store() -> Rc<dyn PrimaryReservationTemplateStore> {
    Rc::new(MemoryPrimaryReservationTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryReservationTemplateStore {
    pub inner: Rc<dyn PrimaryReservationTemplateStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryReservationTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryReservationTemplateStore for CachedPrimaryReservationTemplateStore {
    fn find(&self, code: &str) -> Option<PrimaryReservationTemplate> {
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

    fn save(&self, item: &PrimaryReservationTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryReservationTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_primary_reservation_template_store(
    inner: Rc<dyn PrimaryReservationTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryReservationTemplateStore> {
    Rc::new(CachedPrimaryReservationTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryReservationTemplate>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary reservation templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationTemplateQuery {
    pub title: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryReservationTemplateQuery {
    pub fn matches(&self, item: &PrimaryReservationTemplate) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryReservationTemplate]) -> Vec<PrimaryReservationTemplate> {
        let matched = collections::filter(items, |candidate: &PrimaryReservationTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_reservation_templates(
    store: Rc<dyn PrimaryReservationTemplateStore>,
    query: &PrimaryReservationTemplateQuery,
) -> Vec<PrimaryReservationTemplate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryReservationTemplateStore> = new_memory_primary_reservation_template_store();
        let item = new_primary_reservation_template("code-201", "title-779");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
