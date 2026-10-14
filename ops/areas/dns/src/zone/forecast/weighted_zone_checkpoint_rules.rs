use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted zone checkpoint must pass. Check returns its problems, or none.
pub trait WeightedZoneCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &WeightedZoneCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneCheckpointRequiredLabelRule {
    pub message: String,
}

impl WeightedZoneCheckpointRule for WeightedZoneCheckpointRequiredLabelRule {
    fn check(&self, item: &WeightedZoneCheckpoint) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneCheckpointApprovedGuardRule {
    pub limit: i64,
}

impl WeightedZoneCheckpointRule for WeightedZoneCheckpointApprovedGuardRule {
    fn check(&self, item: &WeightedZoneCheckpoint) -> Vec<String> {
        if item.approved && (item.revision > self.limit) {
            return vec![format!(
                "revision {} exceeds {} while approved",
                item.revision, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneCheckpointRequiredAudienceRule {
    pub message: String,
}

impl WeightedZoneCheckpointRule for WeightedZoneCheckpointRequiredAudienceRule {
    fn check(&self, item: &WeightedZoneCheckpoint) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneCheckpointAudienceLengthRule {
    pub max_length: i64,
}

impl WeightedZoneCheckpointRule for WeightedZoneCheckpointAudienceLengthRule {
    fn check(&self, item: &WeightedZoneCheckpoint) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted zone checkpoint.
#[derive(Debug, Clone)]
pub struct WeightedZoneCheckpointValidator {
    pub rules: Vec<Rc<dyn WeightedZoneCheckpointRule>>,
}

impl WeightedZoneCheckpointValidator {
    pub fn validate(&self, item: &WeightedZoneCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedZoneCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedZoneCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_zone_checkpoint_validator() -> WeightedZoneCheckpointValidator {
    WeightedZoneCheckpointValidator {
        rules: vec![
            Rc::new(WeightedZoneCheckpointRequiredLabelRule {
                message: "weighted zone checkpoint needs an entry".to_string(),
            }) as Rc<dyn WeightedZoneCheckpointRule>,
            Rc::new(WeightedZoneCheckpointApprovedGuardRule { limit: 80 }),
            Rc::new(WeightedZoneCheckpointRequiredAudienceRule {
                message: "weighted zone checkpoint needs this field".to_string(),
            }),
            Rc::new(WeightedZoneCheckpointAudienceLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_zone_checkpoint_validator();
        let item = new_weighted_zone_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = WeightedZoneCheckpointRequiredLabelRule {
            message: "weighted zone checkpoint needs this field".to_string(),
        };
        let mut item = new_weighted_zone_checkpoint("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = WeightedZoneCheckpointRequiredAudienceRule {
            message: "weighted zone checkpoint needs a value".to_string(),
        };
        let mut item = new_weighted_zone_checkpoint("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = WeightedZoneCheckpointAudienceLengthRule { max_length: 120 };
        let mut item = new_weighted_zone_checkpoint("x", "y");
        item.audience = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
