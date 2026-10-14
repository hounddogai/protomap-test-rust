//! Feature flags with rollout strategies.

use std::collections::BTreeMap;
use std::rc::Rc;

/// Strategy decides whether a flag is on for a subject.
pub trait Strategy: std::fmt::Debug {
    fn allows(&self, subject: &str) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrategy {
    pub rollout: i64,
    pub salt: String,
}

impl Strategy for PercentStrategy {
    fn allows(&self, subject: &str) -> bool {
        let bucket = format!("{}{}", self.salt, subject)
            .bytes()
            .fold(0i64, |hash, byte| (hash * 31 + byte as i64) % 2_147_483_647)
            % 100;
        bucket < self.rollout
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowListStrategy {
    pub subjects: Vec<String>,
}

impl Strategy for AllowListStrategy {
    fn allows(&self, subject: &str) -> bool {
        self.subjects.iter().any(|value| value == subject)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefixStrategy {
    pub prefix: String,
}

impl Strategy for PrefixStrategy {
    fn allows(&self, subject: &str) -> bool {
        subject.starts_with(&self.prefix)
    }
}

#[derive(Debug, Clone)]
pub struct AllOf {
    pub parts: Vec<Rc<dyn Strategy>>,
}

impl Strategy for AllOf {
    fn allows(&self, subject: &str) -> bool {
        for part in self.parts.iter() {
            if !part.allows(subject) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct AnyOf {
    pub parts: Vec<Rc<dyn Strategy>>,
}

impl Strategy for AnyOf {
    fn allows(&self, subject: &str) -> bool {
        for part in self.parts.iter() {
            if part.allows(subject) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct Flag {
    pub name: String,
    pub enabled: bool,
    pub strategy: Rc<dyn Strategy>,
}

/// Registry holds the flags of a process by name.
#[derive(Debug, Clone)]
pub struct Registry {
    pub flags: BTreeMap<String, Flag>,
}

impl Registry {
    pub fn register(&mut self, flag: &Flag) {
        self.flags.insert(flag.name.clone(), flag.clone());
    }

    pub fn enabled(&self, name: &str, subject: &str) -> bool {
        let Some(found) = self.flags.get(name).cloned() else {
            return false;
        };
        if !found.enabled {
            return false;
        }
        found.strategy.allows(subject)
    }

    pub fn names(&self) -> Vec<String> {
        self.flags.keys().cloned().collect::<Vec<String>>()
    }
}

pub fn new_registry() -> Registry {
    Registry { flags: BTreeMap::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn honors_allow_lists() {
        let mut registry = new_registry();
        registry.register(&Flag {
            name: "beta".to_string(),
            enabled: true,
            strategy: Rc::new(AllowListStrategy {
                subjects: vec!["tenant-a".to_string()],
            }),
        });
        assert!(registry.enabled("beta", "tenant-a"), "tenant-a");
        assert!(!(registry.enabled("beta", "tenant-b")), "tenant-b");
        assert!(!(registry.enabled("missing", "tenant-a")), "missing flag");
    }
}
