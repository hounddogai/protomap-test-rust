use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps sink rules by code.
pub trait SinkRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SinkRule>;
    fn save(&self, item: &SinkRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SinkRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySinkRuleStore {
    pub items: RefCell<BTreeMap<String, SinkRule>>,
    pub order: RefCell<Vec<String>>,
}

impl SinkRuleStore for MemorySinkRuleStore {
    fn find(&self, code: &str) -> Option<SinkRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SinkRule) {
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

    fn list(&self) -> Vec<SinkRule> {
        let mut out: Vec<SinkRule> = Vec::new();
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

pub fn new_memory_sink_rule_store() -> Rc<dyn SinkRuleStore> {
    Rc::new(MemorySinkRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSinkRuleStore {
    pub inner: Rc<dyn SinkRuleStore>,
    pub cache: RefCell<cache::TtlCache<SinkRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SinkRuleStore for CachedSinkRuleStore {
    fn find(&self, code: &str) -> Option<SinkRule> {
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

    fn save(&self, item: &SinkRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SinkRule> {
        self.inner.list()
    }
}

pub fn new_cached_sink_rule_store(
    inner: Rc<dyn SinkRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SinkRuleStore> {
    Rc::new(CachedSinkRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SinkRule>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects sink rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkRuleQuery {
    pub source: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SinkRuleQuery {
    pub fn matches(&self, item: &SinkRule) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SinkRule]) -> Vec<SinkRule> {
        let matched = collections::filter(items, |candidate: &SinkRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_sink_rules(store: Rc<dyn SinkRuleStore>, query: &SinkRuleQuery) -> Vec<SinkRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SinkRuleStore> = new_memory_sink_rule_store();
        let item = new_sink_rule("code-277", "name-467");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
