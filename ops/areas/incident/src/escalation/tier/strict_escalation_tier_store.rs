use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict escalation tiers by code.
pub trait StrictEscalationTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictEscalationTier>;
    fn save(&self, item: &StrictEscalationTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictEscalationTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictEscalationTierStore {
    pub items: RefCell<BTreeMap<String, StrictEscalationTier>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictEscalationTierStore for MemoryStrictEscalationTierStore {
    fn find(&self, code: &str) -> Option<StrictEscalationTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictEscalationTier) {
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

    fn list(&self) -> Vec<StrictEscalationTier> {
        let mut out: Vec<StrictEscalationTier> = Vec::new();
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

pub fn new_memory_strict_escalation_tier_store() -> Rc<dyn StrictEscalationTierStore> {
    Rc::new(MemoryStrictEscalationTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictEscalationTierStore {
    pub inner: Rc<dyn StrictEscalationTierStore>,
    pub cache: RefCell<cache::TtlCache<StrictEscalationTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictEscalationTierStore for CachedStrictEscalationTierStore {
    fn find(&self, code: &str) -> Option<StrictEscalationTier> {
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

    fn save(&self, item: &StrictEscalationTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictEscalationTier> {
        self.inner.list()
    }
}

pub fn new_cached_strict_escalation_tier_store(
    inner: Rc<dyn StrictEscalationTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictEscalationTierStore> {
    Rc::new(CachedStrictEscalationTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictEscalationTier>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict escalation tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationTierQuery {
    pub segment: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictEscalationTierQuery {
    pub fn matches(&self, item: &StrictEscalationTier) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictEscalationTier]) -> Vec<StrictEscalationTier> {
        let matched = collections::filter(items, |candidate: &StrictEscalationTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_escalation_tiers(
    store: Rc<dyn StrictEscalationTierStore>,
    query: &StrictEscalationTierQuery,
) -> Vec<StrictEscalationTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictEscalationTierStore> = new_memory_strict_escalation_tier_store();
        let item = new_strict_escalation_tier("code-298", "label-835");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
