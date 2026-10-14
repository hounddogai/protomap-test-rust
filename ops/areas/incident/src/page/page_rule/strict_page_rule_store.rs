use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict page rules by code.
pub trait StrictPageRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictPageRule>;
    fn save(&self, item: &StrictPageRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictPageRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictPageRuleStore {
    pub items: RefCell<BTreeMap<String, StrictPageRule>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictPageRuleStore for MemoryStrictPageRuleStore {
    fn find(&self, code: &str) -> Option<StrictPageRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictPageRule) {
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

    fn list(&self) -> Vec<StrictPageRule> {
        let mut out: Vec<StrictPageRule> = Vec::new();
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

pub fn new_memory_strict_page_rule_store() -> Rc<dyn StrictPageRuleStore> {
    Rc::new(MemoryStrictPageRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictPageRuleStore {
    pub inner: Rc<dyn StrictPageRuleStore>,
    pub cache: RefCell<cache::TtlCache<StrictPageRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictPageRuleStore for CachedStrictPageRuleStore {
    fn find(&self, code: &str) -> Option<StrictPageRule> {
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

    fn save(&self, item: &StrictPageRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictPageRule> {
        self.inner.list()
    }
}

pub fn new_cached_strict_page_rule_store(
    inner: Rc<dyn StrictPageRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictPageRuleStore> {
    Rc::new(CachedStrictPageRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictPageRule>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict page rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageRuleQuery {
    pub scope: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictPageRuleQuery {
    pub fn matches(&self, item: &StrictPageRule) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictPageRule]) -> Vec<StrictPageRule> {
        let matched = collections::filter(items, |candidate: &StrictPageRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_page_rules(store: Rc<dyn StrictPageRuleStore>, query: &StrictPageRuleQuery) -> Vec<StrictPageRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictPageRuleStore> = new_memory_strict_page_rule_store();
        let item = new_strict_page_rule("code-958", "title-635");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
