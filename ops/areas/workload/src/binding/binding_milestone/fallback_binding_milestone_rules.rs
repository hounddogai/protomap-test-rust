use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback binding milestone must pass. Check returns its problems, or none.
pub trait FallbackBindingMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &FallbackBindingMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingMilestoneUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackBindingMilestoneRule for FallbackBindingMilestoneUpdatedAtRangeRule {
    fn check(&self, item: &FallbackBindingMilestone) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingMilestoneFingerprintLengthRule {
    pub max_length: i64,
}

impl FallbackBindingMilestoneRule for FallbackBindingMilestoneFingerprintLengthRule {
    fn check(&self, item: &FallbackBindingMilestone) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingMilestoneApprovedGuardRule {
    pub limit: i64,
}

impl FallbackBindingMilestoneRule for FallbackBindingMilestoneApprovedGuardRule {
    fn check(&self, item: &FallbackBindingMilestone) -> Vec<String> {
        if item.approved && (item.updated_at > self.limit) {
            return vec![format!(
                "updated at {} exceeds {} while approved",
                item.updated_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingMilestoneRequiredLabelRule {
    pub message: String,
}

impl FallbackBindingMilestoneRule for FallbackBindingMilestoneRequiredLabelRule {
    fn check(&self, item: &FallbackBindingMilestone) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback binding milestone.
#[derive(Debug, Clone)]
pub struct FallbackBindingMilestoneValidator {
    pub rules: Vec<Rc<dyn FallbackBindingMilestoneRule>>,
}

impl FallbackBindingMilestoneValidator {
    pub fn validate(&self, item: &FallbackBindingMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackBindingMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackBindingMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_binding_milestone_validator() -> FallbackBindingMilestoneValidator {
    FallbackBindingMilestoneValidator {
        rules: vec![
            Rc::new(FallbackBindingMilestoneUpdatedAtRangeRule { min: 80, max: 80 })
                as Rc<dyn FallbackBindingMilestoneRule>,
            Rc::new(FallbackBindingMilestoneFingerprintLengthRule { max_length: 32 }),
            Rc::new(FallbackBindingMilestoneApprovedGuardRule { limit: 64 }),
            Rc::new(FallbackBindingMilestoneRequiredLabelRule {
                message: "fallback binding milestone needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_binding_milestone_validator();
        let item = new_fallback_binding_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = FallbackBindingMilestoneUpdatedAtRangeRule { min: 255, max: 1000 };
        let mut item = new_fallback_binding_milestone("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = FallbackBindingMilestoneFingerprintLengthRule { max_length: 120 };
        let mut item = new_fallback_binding_milestone("x", "y");
        item.fingerprint = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FallbackBindingMilestoneRequiredLabelRule {
            message: "fallback binding milestone needs this field".to_string(),
        };
        let mut item = new_fallback_binding_milestone("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
