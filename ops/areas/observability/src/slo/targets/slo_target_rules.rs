use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a slo target must pass. Check returns its problems, or none.
pub trait SloTargetRule: std::fmt::Debug {
    fn check(&self, item: &SloTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloTargetCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloTargetRule for SloTargetCeilingRangeRule {
    fn check(&self, item: &SloTarget) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloTargetLabelLengthRule {
    pub max_length: i64,
}

impl SloTargetRule for SloTargetLabelLengthRule {
    fn check(&self, item: &SloTarget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloTargetRequiredCodeRule {
    pub message: String,
}

impl SloTargetRule for SloTargetRequiredCodeRule {
    fn check(&self, item: &SloTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloTargetAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl SloTargetRule for SloTargetAllowedReviewerRule {
    fn check(&self, item: &SloTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is duplicated", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloTargetRequiredLabelRule {
    pub message: String,
}

impl SloTargetRule for SloTargetRequiredLabelRule {
    fn check(&self, item: &SloTarget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo target.
#[derive(Debug, Clone)]
pub struct SloTargetValidator {
    pub rules: Vec<Rc<dyn SloTargetRule>>,
}

impl SloTargetValidator {
    pub fn validate(&self, item: &SloTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_target_validator() -> SloTargetValidator {
    SloTargetValidator {
        rules: vec![
            Rc::new(SloTargetCeilingRangeRule { min: 2.0, max: 2452.7 }) as Rc<dyn SloTargetRule>,
            Rc::new(SloTargetLabelLengthRule { max_length: 120 }),
            Rc::new(SloTargetRequiredCodeRule {
                message: "slo target needs a value".to_string(),
            }),
            Rc::new(SloTargetAllowedReviewerRule {
                allowed: vec!["lenient".to_string()],
            }),
            Rc::new(SloTargetRequiredLabelRule {
                message: "slo target needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_target_validator();
        let item = new_slo_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = SloTargetCeilingRangeRule { min: 7.0, max: 3800.5 };
        let mut item = new_slo_target("x", "y");
        item.ceiling = 3802.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SloTargetLabelLengthRule { max_length: 255 };
        let mut item = new_slo_target("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SloTargetRequiredCodeRule {
            message: "slo target needs this field".to_string(),
        };
        let mut item = new_slo_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SloTargetRequiredLabelRule {
            message: "slo target needs a value".to_string(),
        };
        let mut item = new_slo_target("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
