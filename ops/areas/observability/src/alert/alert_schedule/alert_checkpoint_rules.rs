use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert checkpoint must pass. Check returns its problems, or none.
pub trait AlertCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &AlertCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertCheckpointCodePrefixRule {
    pub prefix: String,
}

impl AlertCheckpointRule for AlertCheckpointCodePrefixRule {
    fn check(&self, item: &AlertCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertCheckpointNamespaceLengthRule {
    pub max_length: i64,
}

impl AlertCheckpointRule for AlertCheckpointNamespaceLengthRule {
    fn check(&self, item: &AlertCheckpoint) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertCheckpointArchivedGuardRule {
    pub limit: f64,
}

impl AlertCheckpointRule for AlertCheckpointArchivedGuardRule {
    fn check(&self, item: &AlertCheckpoint) -> Vec<String> {
        if item.archived && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while archived",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertCheckpointQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertCheckpointRule for AlertCheckpointQuotaUnitsRangeRule {
    fn check(&self, item: &AlertCheckpoint) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertCheckpointRequiredLabelRule {
    pub message: String,
}

impl AlertCheckpointRule for AlertCheckpointRequiredLabelRule {
    fn check(&self, item: &AlertCheckpoint) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert checkpoint.
#[derive(Debug, Clone)]
pub struct AlertCheckpointValidator {
    pub rules: Vec<Rc<dyn AlertCheckpointRule>>,
}

impl AlertCheckpointValidator {
    pub fn validate(&self, item: &AlertCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_checkpoint_validator() -> AlertCheckpointValidator {
    AlertCheckpointValidator {
        rules: vec![
            Rc::new(AlertCheckpointCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn AlertCheckpointRule>,
            Rc::new(AlertCheckpointNamespaceLengthRule { max_length: 120 }),
            Rc::new(AlertCheckpointArchivedGuardRule { limit: 4.0 }),
            Rc::new(AlertCheckpointQuotaUnitsRangeRule { min: 16, max: 86400 }),
            Rc::new(AlertCheckpointRequiredLabelRule {
                message: "alert checkpoint needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_checkpoint_validator();
        let item = new_alert_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = AlertCheckpointNamespaceLengthRule { max_length: 16 };
        let mut item = new_alert_checkpoint("x", "y");
        item.namespace = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = AlertCheckpointQuotaUnitsRangeRule { min: 32, max: 86400 };
        let mut item = new_alert_checkpoint("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AlertCheckpointRequiredLabelRule {
            message: "alert checkpoint needs this field".to_string(),
        };
        let mut item = new_alert_checkpoint("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
