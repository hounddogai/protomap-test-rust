//! An in-memory cache with a time to live and a capacity.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::clock;

/// TtlCache keeps up to capacity values for ttl seconds each, evicting the oldest first.
#[derive(Debug, Clone)]
pub struct TtlCache<V> {
    pub entries: BTreeMap<String, V>,
    pub expires: BTreeMap<String, i64>,
    pub order: Vec<String>,
    pub capacity: i64,
    pub ttl: i64,
    pub clock: Rc<dyn clock::Clock>,
}

impl<V: Clone + std::fmt::Debug> TtlCache<V> {
    pub fn get(&mut self, key: &str) -> Option<V> {
        let Some(deadline) = self.expires.get(key).copied() else {
            return None;
        };
        if self.clock.now() > deadline {
            self.entries.remove(key);
            self.expires.remove(key);
            return None;
        }
        let Some(value) = self.entries.get(key).cloned() else {
            return None;
        };
        Some(value)
    }

    pub fn put(&mut self, key: &str, value: &V) {
        if !self.entries.contains_key(key) {
            self.order.push(key.to_string());
        }
        self.entries.insert(key.to_string(), value.clone());
        self.expires.insert(key.to_string(), self.clock.now() + self.ttl);
        while (self.order.len() as i64) > self.capacity && ((self.order.len() as i64) > 0) {
            let oldest = self.order[0].clone();
            self.order = self.order[1 as usize..].to_vec();
            self.entries.remove(&oldest);
            self.expires.remove(&oldest);
        }
    }

    pub fn size(&self) -> i64 {
        self.entries.len() as i64
    }

    pub fn purge_expired(&mut self) -> i64 {
        let now = self.clock.now();
        let mut kept: Vec<String> = Vec::new();
        let mut removed = 0;
        for key in self.order.iter() {
            let deadline = self.expires.get(key).copied().unwrap_or_default();
            if now > deadline {
                self.entries.remove(key);
                self.expires.remove(key);
                removed += 1;
                continue;
            }
            kept.push(key.clone());
        }
        self.order = kept;
        removed
    }
}

pub fn new_ttl_cache<V: Clone + std::fmt::Debug>(capacity: i64, ttl: i64, clock: Rc<dyn clock::Clock>) -> TtlCache<V> {
    TtlCache {
        entries: BTreeMap::new(),
        expires: BTreeMap::new(),
        order: Vec::new(),
        capacity,
        ttl,
        clock: clock.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evicts_the_oldest_entry() {
        let mut cache = new_ttl_cache::<String>(2, 60, Rc::new(clock::FixedClock { at: 1000 }));
        cache.put("a", &"1".to_string());
        cache.put("b", &"2".to_string());
        cache.put("c", &"3".to_string());
        assert_eq!(cache.size(), 2, "size");
    }

    #[test]
    fn forgets_expired_entries() {
        let clock: Rc<dyn clock::Clock> = Rc::new(clock::StepClock {
            current: Cell::new(0),
            step: 100,
        });
        let mut cache = new_ttl_cache::<String>(8, 150, clock.clone());
        cache.put("key", &"value".to_string());
        assert_eq!(cache.purge_expired(), 0, "purged early");
        assert_eq!(cache.purge_expired(), 1, "purged late");
    }
}
