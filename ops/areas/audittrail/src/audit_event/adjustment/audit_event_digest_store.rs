use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps audit event digests by code.
pub trait AuditEventDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AuditEventDigest>;
    fn save(&self, item: &AuditEventDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AuditEventDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAuditEventDigestStore {
    pub items: RefCell<BTreeMap<String, AuditEventDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl AuditEventDigestStore for MemoryAuditEventDigestStore {
    fn find(&self, code: &str) -> Option<AuditEventDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AuditEventDigest) {
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

    fn list(&self) -> Vec<AuditEventDigest> {
        let mut out: Vec<AuditEventDigest> = Vec::new();
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

pub fn new_memory_audit_event_digest_store() -> Rc<dyn AuditEventDigestStore> {
    Rc::new(MemoryAuditEventDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAuditEventDigestStore {
    pub inner: Rc<dyn AuditEventDigestStore>,
    pub cache: RefCell<cache::TtlCache<AuditEventDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AuditEventDigestStore for CachedAuditEventDigestStore {
    fn find(&self, code: &str) -> Option<AuditEventDigest> {
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

    fn save(&self, item: &AuditEventDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AuditEventDigest> {
        self.inner.list()
    }
}

pub fn new_cached_audit_event_digest_store(
    inner: Rc<dyn AuditEventDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AuditEventDigestStore> {
    Rc::new(CachedAuditEventDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AuditEventDigest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects audit event digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventDigestQuery {
    pub scope: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AuditEventDigestQuery {
    pub fn matches(&self, item: &AuditEventDigest) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AuditEventDigest]) -> Vec<AuditEventDigest> {
        let matched = collections::filter(items, |candidate: &AuditEventDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_audit_event_digests(
    store: Rc<dyn AuditEventDigestStore>,
    query: &AuditEventDigestQuery,
) -> Vec<AuditEventDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AuditEventDigestStore> = new_memory_audit_event_digest_store();
        let item = new_audit_event_digest("code-848", "title-417");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
