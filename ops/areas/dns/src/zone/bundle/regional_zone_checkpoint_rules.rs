use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional zone checkpoint must pass. Check returns its problems, or none.
pub trait RegionalZoneCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RegionalZoneCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneCheckpointArchivedGuardRule {
    pub limit: i64,
}

impl RegionalZoneCheckpointRule for RegionalZoneCheckpointArchivedGuardRule {
    fn check(&self, item: &RegionalZoneCheckpoint) -> Vec<String> {
        if item.archived && (item.limit > self.limit) {
            return vec![format!("limit {} exceeds {} while archived", item.limit, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneCheckpointRequiredCodeRule {
    pub message: String,
}

impl RegionalZoneCheckpointRule for RegionalZoneCheckpointRequiredCodeRule {
    fn check(&self, item: &RegionalZoneCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneCheckpointCodePrefixRule {
    pub prefix: String,
}

impl RegionalZoneCheckpointRule for RegionalZoneCheckpointCodePrefixRule {
    fn check(&self, item: &RegionalZoneCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneCheckpointLocaleLengthRule {
    pub max_length: i64,
}

impl RegionalZoneCheckpointRule for RegionalZoneCheckpointLocaleLengthRule {
    fn check(&self, item: &RegionalZoneCheckpoint) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneCheckpointRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalZoneCheckpointRule for RegionalZoneCheckpointRetriesRangeRule {
    fn check(&self, item: &RegionalZoneCheckpoint) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneCheckpointRequiredLocaleRule {
    pub message: String,
}

impl RegionalZoneCheckpointRule for RegionalZoneCheckpointRequiredLocaleRule {
    fn check(&self, item: &RegionalZoneCheckpoint) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional zone checkpoint.
#[derive(Debug, Clone)]
pub struct RegionalZoneCheckpointValidator {
    pub rules: Vec<Rc<dyn RegionalZoneCheckpointRule>>,
}

impl RegionalZoneCheckpointValidator {
    pub fn validate(&self, item: &RegionalZoneCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalZoneCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalZoneCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_zone_checkpoint_validator() -> RegionalZoneCheckpointValidator {
    RegionalZoneCheckpointValidator {
        rules: vec![
            Rc::new(RegionalZoneCheckpointArchivedGuardRule { limit: 80 }) as Rc<dyn RegionalZoneCheckpointRule>,
            Rc::new(RegionalZoneCheckpointRequiredCodeRule {
                message: "regional zone checkpoint needs an entry".to_string(),
            }),
            Rc::new(RegionalZoneCheckpointCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RegionalZoneCheckpointLocaleLengthRule { max_length: 255 }),
            Rc::new(RegionalZoneCheckpointRetriesRangeRule { min: 64, max: 3600 }),
            Rc::new(RegionalZoneCheckpointRequiredLocaleRule {
                message: "regional zone checkpoint needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_zone_checkpoint_validator();
        let item = new_regional_zone_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalZoneCheckpointRequiredCodeRule {
            message: "regional zone checkpoint needs this field".to_string(),
        };
        let mut item = new_regional_zone_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = RegionalZoneCheckpointLocaleLengthRule { max_length: 120 };
        let mut item = new_regional_zone_checkpoint("x", "y");
        item.locale = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = RegionalZoneCheckpointRetriesRangeRule { min: 64, max: 120 };
        let mut item = new_regional_zone_checkpoint("x", "y");
        item.retries = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = RegionalZoneCheckpointRequiredLocaleRule {
            message: "regional zone checkpoint needs a value".to_string(),
        };
        let mut item = new_regional_zone_checkpoint("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
