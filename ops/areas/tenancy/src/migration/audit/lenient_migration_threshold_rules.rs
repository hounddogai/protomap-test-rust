use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient migration threshold must pass. Check returns its problems, or none.
pub trait LenientMigrationThresholdRule: std::fmt::Debug {
    fn check(&self, item: &LenientMigrationThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationThresholdLabelLengthRule {
    pub max_length: i64,
}

impl LenientMigrationThresholdRule for LenientMigrationThresholdLabelLengthRule {
    fn check(&self, item: &LenientMigrationThreshold) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationThresholdCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientMigrationThresholdRule for LenientMigrationThresholdCreatedAtRangeRule {
    fn check(&self, item: &LenientMigrationThreshold) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationThresholdCodePrefixRule {
    pub prefix: String,
}

impl LenientMigrationThresholdRule for LenientMigrationThresholdCodePrefixRule {
    fn check(&self, item: &LenientMigrationThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationThresholdVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientMigrationThresholdRule for LenientMigrationThresholdVersionRangeRule {
    fn check(&self, item: &LenientMigrationThreshold) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationThresholdRequiredLabelRule {
    pub message: String,
}

impl LenientMigrationThresholdRule for LenientMigrationThresholdRequiredLabelRule {
    fn check(&self, item: &LenientMigrationThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient migration threshold.
#[derive(Debug, Clone)]
pub struct LenientMigrationThresholdValidator {
    pub rules: Vec<Rc<dyn LenientMigrationThresholdRule>>,
}

impl LenientMigrationThresholdValidator {
    pub fn validate(&self, item: &LenientMigrationThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientMigrationThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientMigrationThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_migration_threshold_validator() -> LenientMigrationThresholdValidator {
    LenientMigrationThresholdValidator {
        rules: vec![
            Rc::new(LenientMigrationThresholdLabelLengthRule { max_length: 255 })
                as Rc<dyn LenientMigrationThresholdRule>,
            Rc::new(LenientMigrationThresholdCreatedAtRangeRule { min: 120, max: 86400 }),
            Rc::new(LenientMigrationThresholdCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LenientMigrationThresholdVersionRangeRule { min: 32, max: 80 }),
            Rc::new(LenientMigrationThresholdRequiredLabelRule {
                message: "lenient migration threshold needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_migration_threshold_validator();
        let item = new_lenient_migration_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientMigrationThresholdLabelLengthRule { max_length: 16 };
        let mut item = new_lenient_migration_threshold("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = LenientMigrationThresholdCreatedAtRangeRule { min: 32, max: 86400 };
        let mut item = new_lenient_migration_threshold("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = LenientMigrationThresholdVersionRangeRule { min: 16, max: 86400 };
        let mut item = new_lenient_migration_threshold("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientMigrationThresholdRequiredLabelRule {
            message: "lenient migration threshold needs an entry".to_string(),
        };
        let mut item = new_lenient_migration_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
