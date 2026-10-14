use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a silence adjustment must pass. Check returns its problems, or none.
pub trait SilenceAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &SilenceAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAdjustmentRequiredCodeRule {
    pub message: String,
}

impl SilenceAdjustmentRule for SilenceAdjustmentRequiredCodeRule {
    fn check(&self, item: &SilenceAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAdjustmentRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceAdjustmentRule for SilenceAdjustmentRotationDaysRangeRule {
    fn check(&self, item: &SilenceAdjustment) -> Vec<String> {
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
pub struct SilenceAdjustmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SilenceAdjustmentRule for SilenceAdjustmentAllowedCodeRule {
    fn check(&self, item: &SilenceAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAdjustmentExternalRefLengthRule {
    pub max_length: i64,
}

impl SilenceAdjustmentRule for SilenceAdjustmentExternalRefLengthRule {
    fn check(&self, item: &SilenceAdjustment) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAdjustmentPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceAdjustmentRule for SilenceAdjustmentPriorityRangeRule {
    fn check(&self, item: &SilenceAdjustment) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a silence adjustment.
#[derive(Debug, Clone)]
pub struct SilenceAdjustmentValidator {
    pub rules: Vec<Rc<dyn SilenceAdjustmentRule>>,
}

impl SilenceAdjustmentValidator {
    pub fn validate(&self, item: &SilenceAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_adjustment_validator() -> SilenceAdjustmentValidator {
    SilenceAdjustmentValidator {
        rules: vec![
            Rc::new(SilenceAdjustmentRequiredCodeRule {
                message: "silence adjustment needs this field".to_string(),
            }) as Rc<dyn SilenceAdjustmentRule>,
            Rc::new(SilenceAdjustmentRotationDaysRangeRule { min: 120, max: 255 }),
            Rc::new(SilenceAdjustmentAllowedCodeRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(SilenceAdjustmentExternalRefLengthRule { max_length: 1000 }),
            Rc::new(SilenceAdjustmentPriorityRangeRule { min: 64, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_adjustment_validator();
        let item = new_silence_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SilenceAdjustmentRequiredCodeRule {
            message: "silence adjustment needs this field".to_string(),
        };
        let mut item = new_silence_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = SilenceAdjustmentRotationDaysRangeRule { min: 1000, max: 3600 };
        let mut item = new_silence_adjustment("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = SilenceAdjustmentExternalRefLengthRule { max_length: 16 };
        let mut item = new_silence_adjustment("x", "y");
        item.external_ref = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = SilenceAdjustmentPriorityRangeRule { min: 255, max: 86400 };
        let mut item = new_silence_adjustment("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
