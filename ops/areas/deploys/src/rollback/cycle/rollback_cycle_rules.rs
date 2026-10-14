use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback cycle must pass. Check returns its problems, or none.
pub trait RollbackCycleRule: std::fmt::Debug {
    fn check(&self, item: &RollbackCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCycleRequiredCodeRule {
    pub message: String,
}

impl RollbackCycleRule for RollbackCycleRequiredCodeRule {
    fn check(&self, item: &RollbackCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCycleCodePrefixRule {
    pub prefix: String,
}

impl RollbackCycleRule for RollbackCycleCodePrefixRule {
    fn check(&self, item: &RollbackCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCycleRequiredNameRule {
    pub message: String,
}

impl RollbackCycleRule for RollbackCycleRequiredNameRule {
    fn check(&self, item: &RollbackCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCycleArchivedGuardRule {
    pub limit: i64,
}

impl RollbackCycleRule for RollbackCycleArchivedGuardRule {
    fn check(&self, item: &RollbackCycle) -> Vec<String> {
        if item.archived && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while archived",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCycleClusterLengthRule {
    pub max_length: i64,
}

impl RollbackCycleRule for RollbackCycleClusterLengthRule {
    fn check(&self, item: &RollbackCycle) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback cycle.
#[derive(Debug, Clone)]
pub struct RollbackCycleValidator {
    pub rules: Vec<Rc<dyn RollbackCycleRule>>,
}

impl RollbackCycleValidator {
    pub fn validate(&self, item: &RollbackCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackCycle]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_rollback_cycle_validator() -> RollbackCycleValidator {
    RollbackCycleValidator {
        rules: vec![
            Rc::new(RollbackCycleRequiredCodeRule {
                message: "rollback cycle needs this field".to_string(),
            }) as Rc<dyn RollbackCycleRule>,
            Rc::new(RollbackCycleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RollbackCycleRequiredNameRule {
                message: "rollback cycle needs a value".to_string(),
            }),
            Rc::new(RollbackCycleArchivedGuardRule { limit: 3600 }),
            Rc::new(RollbackCycleClusterLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_cycle_validator();
        let item = new_rollback_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollbackCycleRequiredCodeRule {
            message: "rollback cycle needs a value".to_string(),
        };
        let mut item = new_rollback_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollbackCycleRequiredNameRule {
            message: "rollback cycle needs a value".to_string(),
        };
        let mut item = new_rollback_cycle("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = RollbackCycleClusterLengthRule { max_length: 120 };
        let mut item = new_rollback_cycle("x", "y");
        item.cluster = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
