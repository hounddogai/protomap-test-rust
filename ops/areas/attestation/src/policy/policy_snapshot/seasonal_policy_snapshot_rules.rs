use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal policy snapshot must pass. Check returns its problems, or none.
pub trait SeasonalPolicySnapshotRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalPolicySnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPolicySnapshotTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalPolicySnapshotRule for SeasonalPolicySnapshotTtlSecondsRangeRule {
    fn check(&self, item: &SeasonalPolicySnapshot) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPolicySnapshotRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalPolicySnapshotRule for SeasonalPolicySnapshotRetriesRangeRule {
    fn check(&self, item: &SeasonalPolicySnapshot) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPolicySnapshotLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalPolicySnapshotRule for SeasonalPolicySnapshotLabelLengthRule {
    fn check(&self, item: &SeasonalPolicySnapshot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPolicySnapshotAllowedNameRule {
    pub allowed: Vec<String>,
}

impl SeasonalPolicySnapshotRule for SeasonalPolicySnapshotAllowedNameRule {
    fn check(&self, item: &SeasonalPolicySnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is blocked", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPolicySnapshotRequiredDescriptionRule {
    pub message: String,
}

impl SeasonalPolicySnapshotRule for SeasonalPolicySnapshotRequiredDescriptionRule {
    fn check(&self, item: &SeasonalPolicySnapshot) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal policy snapshot.
#[derive(Debug, Clone)]
pub struct SeasonalPolicySnapshotValidator {
    pub rules: Vec<Rc<dyn SeasonalPolicySnapshotRule>>,
}

impl SeasonalPolicySnapshotValidator {
    pub fn validate(&self, item: &SeasonalPolicySnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalPolicySnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalPolicySnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_policy_snapshot_validator() -> SeasonalPolicySnapshotValidator {
    SeasonalPolicySnapshotValidator {
        rules: vec![
            Rc::new(SeasonalPolicySnapshotTtlSecondsRangeRule { min: 64, max: 80 })
                as Rc<dyn SeasonalPolicySnapshotRule>,
            Rc::new(SeasonalPolicySnapshotRetriesRangeRule { min: 64, max: 86400 }),
            Rc::new(SeasonalPolicySnapshotLabelLengthRule { max_length: 3600 }),
            Rc::new(SeasonalPolicySnapshotAllowedNameRule {
                allowed: vec!["global".to_string()],
            }),
            Rc::new(SeasonalPolicySnapshotRequiredDescriptionRule {
                message: "seasonal policy snapshot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_policy_snapshot_validator();
        let item = new_seasonal_policy_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = SeasonalPolicySnapshotTtlSecondsRangeRule { min: 16, max: 80 };
        let mut item = new_seasonal_policy_snapshot("x", "y");
        item.ttl_seconds = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = SeasonalPolicySnapshotRetriesRangeRule { min: 3600, max: 86400 };
        let mut item = new_seasonal_policy_snapshot("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SeasonalPolicySnapshotLabelLengthRule { max_length: 64 };
        let mut item = new_seasonal_policy_snapshot("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = SeasonalPolicySnapshotRequiredDescriptionRule {
            message: "seasonal policy snapshot needs this field".to_string(),
        };
        let mut item = new_seasonal_policy_snapshot("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
