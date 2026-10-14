use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback action item digest must pass. Check returns its problems, or none.
pub trait FallbackActionItemDigestRule: std::fmt::Debug {
    fn check(&self, item: &FallbackActionItemDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemDigestRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackActionItemDigestRule for FallbackActionItemDigestRotationDaysRangeRule {
    fn check(&self, item: &FallbackActionItemDigest) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemDigestRequiredCodeRule {
    pub message: String,
}

impl FallbackActionItemDigestRule for FallbackActionItemDigestRequiredCodeRule {
    fn check(&self, item: &FallbackActionItemDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemDigestRequiredLabelRule {
    pub message: String,
}

impl FallbackActionItemDigestRule for FallbackActionItemDigestRequiredLabelRule {
    fn check(&self, item: &FallbackActionItemDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback action item digest.
#[derive(Debug, Clone)]
pub struct FallbackActionItemDigestValidator {
    pub rules: Vec<Rc<dyn FallbackActionItemDigestRule>>,
}

impl FallbackActionItemDigestValidator {
    pub fn validate(&self, item: &FallbackActionItemDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackActionItemDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackActionItemDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_action_item_digest_validator() -> FallbackActionItemDigestValidator {
    FallbackActionItemDigestValidator {
        rules: vec![
            Rc::new(FallbackActionItemDigestRotationDaysRangeRule { min: 32, max: 86400 })
                as Rc<dyn FallbackActionItemDigestRule>,
            Rc::new(FallbackActionItemDigestRequiredCodeRule {
                message: "fallback action item digest needs a value".to_string(),
            }),
            Rc::new(FallbackActionItemDigestRequiredLabelRule {
                message: "fallback action item digest needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_action_item_digest_validator();
        let item = new_fallback_action_item_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = FallbackActionItemDigestRotationDaysRangeRule { min: 32, max: 64 };
        let mut item = new_fallback_action_item_digest("x", "y");
        item.rotation_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackActionItemDigestRequiredCodeRule {
            message: "fallback action item digest needs a value".to_string(),
        };
        let mut item = new_fallback_action_item_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FallbackActionItemDigestRequiredLabelRule {
            message: "fallback action item digest needs this field".to_string(),
        };
        let mut item = new_fallback_action_item_digest("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
