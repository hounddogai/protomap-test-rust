use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal escalation agreements by code.
pub trait SeasonalEscalationAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalEscalationAgreement>;
    fn save(&self, item: &SeasonalEscalationAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalEscalationAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalEscalationAgreementStore {
    pub items: RefCell<BTreeMap<String, SeasonalEscalationAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalEscalationAgreementStore for MemorySeasonalEscalationAgreementStore {
    fn find(&self, code: &str) -> Option<SeasonalEscalationAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalEscalationAgreement) {
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

    fn list(&self) -> Vec<SeasonalEscalationAgreement> {
        let mut out: Vec<SeasonalEscalationAgreement> = Vec::new();
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

pub fn new_memory_seasonal_escalation_agreement_store() -> Rc<dyn SeasonalEscalationAgreementStore> {
    Rc::new(MemorySeasonalEscalationAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalEscalationAgreementStore {
    pub inner: Rc<dyn SeasonalEscalationAgreementStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalEscalationAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalEscalationAgreementStore for CachedSeasonalEscalationAgreementStore {
    fn find(&self, code: &str) -> Option<SeasonalEscalationAgreement> {
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

    fn save(&self, item: &SeasonalEscalationAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalEscalationAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_escalation_agreement_store(
    inner: Rc<dyn SeasonalEscalationAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalEscalationAgreementStore> {
    Rc::new(CachedSeasonalEscalationAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalEscalationAgreement>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal escalation agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEscalationAgreementQuery {
    pub country: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalEscalationAgreementQuery {
    pub fn matches(&self, item: &SeasonalEscalationAgreement) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalEscalationAgreement]) -> Vec<SeasonalEscalationAgreement> {
        let matched = collections::filter(items, |candidate: &SeasonalEscalationAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_escalation_agreements(
    store: Rc<dyn SeasonalEscalationAgreementStore>,
    query: &SeasonalEscalationAgreementQuery,
) -> Vec<SeasonalEscalationAgreement> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalEscalationAgreementStore> = new_memory_seasonal_escalation_agreement_store();
        let item = new_seasonal_escalation_agreement("code-711", "name-873");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
