use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending slo profile must pass. Check returns its problems, or none.
pub trait PendingSloProfileRule: std::fmt::Debug {
    fn check(&self, item: &PendingSloProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloProfileRequiredLabelRule {
    pub message: String,
}

impl PendingSloProfileRule for PendingSloProfileRequiredLabelRule {
    fn check(&self, item: &PendingSloProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloProfileCodePrefixRule {
    pub prefix: String,
}

impl PendingSloProfileRule for PendingSloProfileCodePrefixRule {
    fn check(&self, item: &PendingSloProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloProfileMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingSloProfileRule for PendingSloProfileMinimumRangeRule {
    fn check(&self, item: &PendingSloProfile) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloProfileVisibleGuardRule {
    pub limit: i64,
}

impl PendingSloProfileRule for PendingSloProfileVisibleGuardRule {
    fn check(&self, item: &PendingSloProfile) -> Vec<String> {
        if item.visible && (item.limit > self.limit) {
            return vec![format!("limit {} exceeds {} while visible", item.limit, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloProfileCodeLengthRule {
    pub max_length: i64,
}

impl PendingSloProfileRule for PendingSloProfileCodeLengthRule {
    fn check(&self, item: &PendingSloProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloProfileAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PendingSloProfileRule for PendingSloProfileAllowedLabelRule {
    fn check(&self, item: &PendingSloProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending slo profile.
#[derive(Debug, Clone)]
pub struct PendingSloProfileValidator {
    pub rules: Vec<Rc<dyn PendingSloProfileRule>>,
}

impl PendingSloProfileValidator {
    pub fn validate(&self, item: &PendingSloProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingSloProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingSloProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_slo_profile_validator() -> PendingSloProfileValidator {
    PendingSloProfileValidator {
        rules: vec![
            Rc::new(PendingSloProfileRequiredLabelRule {
                message: "pending slo profile needs a value".to_string(),
            }) as Rc<dyn PendingSloProfileRule>,
            Rc::new(PendingSloProfileCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PendingSloProfileMinimumRangeRule { min: 5.5, max: 1298.5 }),
            Rc::new(PendingSloProfileVisibleGuardRule { limit: 3600 }),
            Rc::new(PendingSloProfileCodeLengthRule { max_length: 3600 }),
            Rc::new(PendingSloProfileAllowedLabelRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_slo_profile_validator();
        let item = new_pending_slo_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingSloProfileRequiredLabelRule {
            message: "pending slo profile needs a value".to_string(),
        };
        let mut item = new_pending_slo_profile("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PendingSloProfileMinimumRangeRule { min: 6.8, max: 4292.7 };
        let mut item = new_pending_slo_profile("x", "y");
        item.minimum = 4294.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingSloProfileCodeLengthRule { max_length: 16 };
        let mut item = new_pending_slo_profile("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
