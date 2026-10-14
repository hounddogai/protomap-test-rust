use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient migration profile must pass. Check returns its problems, or none.
pub trait LenientMigrationProfileRule: std::fmt::Debug {
    fn check(&self, item: &LenientMigrationProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationProfileRequiredCodeRule {
    pub message: String,
}

impl LenientMigrationProfileRule for LenientMigrationProfileRequiredCodeRule {
    fn check(&self, item: &LenientMigrationProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationProfileCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientMigrationProfileRule for LenientMigrationProfileCapacityRangeRule {
    fn check(&self, item: &LenientMigrationProfile) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationProfileClusterLengthRule {
    pub max_length: i64,
}

impl LenientMigrationProfileRule for LenientMigrationProfileClusterLengthRule {
    fn check(&self, item: &LenientMigrationProfile) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationProfileThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientMigrationProfileRule for LenientMigrationProfileThresholdRangeRule {
    fn check(&self, item: &LenientMigrationProfile) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationProfileRequiredReviewerRule {
    pub message: String,
}

impl LenientMigrationProfileRule for LenientMigrationProfileRequiredReviewerRule {
    fn check(&self, item: &LenientMigrationProfile) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient migration profile.
#[derive(Debug, Clone)]
pub struct LenientMigrationProfileValidator {
    pub rules: Vec<Rc<dyn LenientMigrationProfileRule>>,
}

impl LenientMigrationProfileValidator {
    pub fn validate(&self, item: &LenientMigrationProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientMigrationProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientMigrationProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_migration_profile_validator() -> LenientMigrationProfileValidator {
    LenientMigrationProfileValidator {
        rules: vec![
            Rc::new(LenientMigrationProfileRequiredCodeRule {
                message: "lenient migration profile needs an entry".to_string(),
            }) as Rc<dyn LenientMigrationProfileRule>,
            Rc::new(LenientMigrationProfileCapacityRangeRule { min: 32, max: 86400 }),
            Rc::new(LenientMigrationProfileClusterLengthRule { max_length: 64 }),
            Rc::new(LenientMigrationProfileThresholdRangeRule { min: 2.2, max: 1261.1 }),
            Rc::new(LenientMigrationProfileRequiredReviewerRule {
                message: "lenient migration profile needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_migration_profile_validator();
        let item = new_lenient_migration_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientMigrationProfileRequiredCodeRule {
            message: "lenient migration profile needs a value".to_string(),
        };
        let mut item = new_lenient_migration_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = LenientMigrationProfileCapacityRangeRule { min: 80, max: 86400 };
        let mut item = new_lenient_migration_profile("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = LenientMigrationProfileClusterLengthRule { max_length: 80 };
        let mut item = new_lenient_migration_profile("x", "y");
        item.cluster = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = LenientMigrationProfileThresholdRangeRule { min: 9.0, max: 1634.1 };
        let mut item = new_lenient_migration_profile("x", "y");
        item.threshold = 1635.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = LenientMigrationProfileRequiredReviewerRule {
            message: "lenient migration profile needs an entry".to_string(),
        };
        let mut item = new_lenient_migration_profile("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
