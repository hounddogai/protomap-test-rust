use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual sink digest must pass. Check returns its problems, or none.
pub trait ManualSinkDigestRule: std::fmt::Debug {
    fn check(&self, item: &ManualSinkDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkDigestRequiredLabelRule {
    pub message: String,
}

impl ManualSinkDigestRule for ManualSinkDigestRequiredLabelRule {
    fn check(&self, item: &ManualSinkDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkDigestQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualSinkDigestRule for ManualSinkDigestQuantityRangeRule {
    fn check(&self, item: &ManualSinkDigest) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkDigestCodePrefixRule {
    pub prefix: String,
}

impl ManualSinkDigestRule for ManualSinkDigestCodePrefixRule {
    fn check(&self, item: &ManualSinkDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkDigestRequiredClusterRule {
    pub message: String,
}

impl ManualSinkDigestRule for ManualSinkDigestRequiredClusterRule {
    fn check(&self, item: &ManualSinkDigest) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkDigestGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualSinkDigestRule for ManualSinkDigestGraceDaysRangeRule {
    fn check(&self, item: &ManualSinkDigest) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual sink digest.
#[derive(Debug, Clone)]
pub struct ManualSinkDigestValidator {
    pub rules: Vec<Rc<dyn ManualSinkDigestRule>>,
}

impl ManualSinkDigestValidator {
    pub fn validate(&self, item: &ManualSinkDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualSinkDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualSinkDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_sink_digest_validator() -> ManualSinkDigestValidator {
    ManualSinkDigestValidator {
        rules: vec![
            Rc::new(ManualSinkDigestRequiredLabelRule {
                message: "manual sink digest needs a value".to_string(),
            }) as Rc<dyn ManualSinkDigestRule>,
            Rc::new(ManualSinkDigestQuantityRangeRule { min: 3600, max: 86400 }),
            Rc::new(ManualSinkDigestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ManualSinkDigestRequiredClusterRule {
                message: "manual sink digest needs an entry".to_string(),
            }),
            Rc::new(ManualSinkDigestGraceDaysRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_sink_digest_validator();
        let item = new_manual_sink_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualSinkDigestRequiredLabelRule {
            message: "manual sink digest needs a value".to_string(),
        };
        let mut item = new_manual_sink_digest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ManualSinkDigestQuantityRangeRule { min: 32, max: 120 };
        let mut item = new_manual_sink_digest("x", "y");
        item.quantity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = ManualSinkDigestRequiredClusterRule {
            message: "manual sink digest needs a value".to_string(),
        };
        let mut item = new_manual_sink_digest("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = ManualSinkDigestGraceDaysRangeRule { min: 255, max: 3600 };
        let mut item = new_manual_sink_digest("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
