use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback tenant rules by code.
pub trait FallbackTenantRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackTenantRule>;
    fn save(&self, item: &FallbackTenantRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackTenantRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackTenantRuleStore {
    pub items: RefCell<BTreeMap<String, FallbackTenantRule>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackTenantRuleStore for MemoryFallbackTenantRuleStore {
    fn find(&self, code: &str) -> Option<FallbackTenantRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackTenantRule) {
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

    fn list(&self) -> Vec<FallbackTenantRule> {
        let mut out: Vec<FallbackTenantRule> = Vec::new();
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

pub fn new_memory_fallback_tenant_rule_store() -> Rc<dyn FallbackTenantRuleStore> {
    Rc::new(MemoryFallbackTenantRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackTenantRuleStore {
    pub inner: Rc<dyn FallbackTenantRuleStore>,
    pub cache: RefCell<cache::TtlCache<FallbackTenantRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackTenantRuleStore for CachedFallbackTenantRuleStore {
    fn find(&self, code: &str) -> Option<FallbackTenantRule> {
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

    fn save(&self, item: &FallbackTenantRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackTenantRule> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_tenant_rule_store(
    inner: Rc<dyn FallbackTenantRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackTenantRuleStore> {
    Rc::new(CachedFallbackTenantRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackTenantRule>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback tenant rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantRuleQuery {
    pub channel: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackTenantRuleQuery {
    pub fn matches(&self, item: &FallbackTenantRule) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackTenantRule]) -> Vec<FallbackTenantRule> {
        let matched = collections::filter(items, |candidate: &FallbackTenantRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_tenant_rules(
    store: Rc<dyn FallbackTenantRuleStore>,
    query: &FallbackTenantRuleQuery,
) -> Vec<FallbackTenantRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackTenantRuleStore> = new_memory_fallback_tenant_rule_store();
        let item = new_fallback_tenant_rule("code-142", "title-734");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
