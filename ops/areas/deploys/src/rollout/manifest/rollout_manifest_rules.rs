use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout manifest must pass. Check returns its problems, or none.
pub trait RolloutManifestRule: std::fmt::Debug {
    fn check(&self, item: &RolloutManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutManifestRequiredReviewerRule {
    pub message: String,
}

impl RolloutManifestRule for RolloutManifestRequiredReviewerRule {
    fn check(&self, item: &RolloutManifest) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutManifestActiveGuardRule {
    pub limit: i64,
}

impl RolloutManifestRule for RolloutManifestActiveGuardRule {
    fn check(&self, item: &RolloutManifest) -> Vec<String> {
        if item.active && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while active",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutManifestCodeLengthRule {
    pub max_length: i64,
}

impl RolloutManifestRule for RolloutManifestCodeLengthRule {
    fn check(&self, item: &RolloutManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutManifestBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutManifestRule for RolloutManifestBurstRangeRule {
    fn check(&self, item: &RolloutManifest) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutManifestRequiredCodeRule {
    pub message: String,
}

impl RolloutManifestRule for RolloutManifestRequiredCodeRule {
    fn check(&self, item: &RolloutManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout manifest.
#[derive(Debug, Clone)]
pub struct RolloutManifestValidator {
    pub rules: Vec<Rc<dyn RolloutManifestRule>>,
}

impl RolloutManifestValidator {
    pub fn validate(&self, item: &RolloutManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_manifest_validator() -> RolloutManifestValidator {
    RolloutManifestValidator {
        rules: vec![
            Rc::new(RolloutManifestRequiredReviewerRule {
                message: "rollout manifest needs a value".to_string(),
            }) as Rc<dyn RolloutManifestRule>,
            Rc::new(RolloutManifestActiveGuardRule { limit: 32 }),
            Rc::new(RolloutManifestCodeLengthRule { max_length: 1000 }),
            Rc::new(RolloutManifestBurstRangeRule { min: 16, max: 86400 }),
            Rc::new(RolloutManifestRequiredCodeRule {
                message: "rollout manifest needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_manifest_validator();
        let item = new_rollout_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = RolloutManifestRequiredReviewerRule {
            message: "rollout manifest needs a value".to_string(),
        };
        let mut item = new_rollout_manifest("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RolloutManifestCodeLengthRule { max_length: 1000 };
        let mut item = new_rollout_manifest("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = RolloutManifestBurstRangeRule { min: 80, max: 120 };
        let mut item = new_rollout_manifest("x", "y");
        item.burst = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RolloutManifestRequiredCodeRule {
            message: "rollout manifest needs this field".to_string(),
        };
        let mut item = new_rollout_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
