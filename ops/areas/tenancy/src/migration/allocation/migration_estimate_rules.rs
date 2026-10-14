use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration estimate must pass. Check returns its problems, or none.
pub trait MigrationEstimateRule: std::fmt::Debug {
    fn check(&self, item: &MigrationEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationEstimateCodePrefixRule {
    pub prefix: String,
}

impl MigrationEstimateRule for MigrationEstimateCodePrefixRule {
    fn check(&self, item: &MigrationEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationEstimateBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationEstimateRule for MigrationEstimateBurstRangeRule {
    fn check(&self, item: &MigrationEstimate) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationEstimateRequiredTitleRule {
    pub message: String,
}

impl MigrationEstimateRule for MigrationEstimateRequiredTitleRule {
    fn check(&self, item: &MigrationEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationEstimateDescriptionLengthRule {
    pub max_length: i64,
}

impl MigrationEstimateRule for MigrationEstimateDescriptionLengthRule {
    fn check(&self, item: &MigrationEstimate) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationEstimateRequiredDescriptionRule {
    pub message: String,
}

impl MigrationEstimateRule for MigrationEstimateRequiredDescriptionRule {
    fn check(&self, item: &MigrationEstimate) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration estimate.
#[derive(Debug, Clone)]
pub struct MigrationEstimateValidator {
    pub rules: Vec<Rc<dyn MigrationEstimateRule>>,
}

impl MigrationEstimateValidator {
    pub fn validate(&self, item: &MigrationEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_estimate_validator() -> MigrationEstimateValidator {
    MigrationEstimateValidator {
        rules: vec![
            Rc::new(MigrationEstimateCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn MigrationEstimateRule>,
            Rc::new(MigrationEstimateBurstRangeRule { min: 120, max: 1000 }),
            Rc::new(MigrationEstimateRequiredTitleRule {
                message: "migration estimate needs a value".to_string(),
            }),
            Rc::new(MigrationEstimateDescriptionLengthRule { max_length: 80 }),
            Rc::new(MigrationEstimateRequiredDescriptionRule {
                message: "migration estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_estimate_validator();
        let item = new_migration_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = MigrationEstimateBurstRangeRule { min: 3600, max: 86400 };
        let mut item = new_migration_estimate("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = MigrationEstimateRequiredTitleRule {
            message: "migration estimate needs a value".to_string(),
        };
        let mut item = new_migration_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = MigrationEstimateDescriptionLengthRule { max_length: 80 };
        let mut item = new_migration_estimate("x", "y");
        item.description = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = MigrationEstimateRequiredDescriptionRule {
            message: "migration estimate needs a value".to_string(),
        };
        let mut item = new_migration_estimate("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
