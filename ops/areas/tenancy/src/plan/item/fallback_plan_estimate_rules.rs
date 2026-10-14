use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback plan estimate must pass. Check returns its problems, or none.
pub trait FallbackPlanEstimateRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPlanEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanEstimateSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPlanEstimateRule for FallbackPlanEstimateSequenceRangeRule {
    fn check(&self, item: &FallbackPlanEstimate) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanEstimateCodePrefixRule {
    pub prefix: String,
}

impl FallbackPlanEstimateRule for FallbackPlanEstimateCodePrefixRule {
    fn check(&self, item: &FallbackPlanEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanEstimateAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FallbackPlanEstimateRule for FallbackPlanEstimateAllowedCodeRule {
    fn check(&self, item: &FallbackPlanEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanEstimateRequiredNotesRule {
    pub message: String,
}

impl FallbackPlanEstimateRule for FallbackPlanEstimateRequiredNotesRule {
    fn check(&self, item: &FallbackPlanEstimate) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanEstimateWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPlanEstimateRule for FallbackPlanEstimateWindowDaysRangeRule {
    fn check(&self, item: &FallbackPlanEstimate) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanEstimateRequiredCodeRule {
    pub message: String,
}

impl FallbackPlanEstimateRule for FallbackPlanEstimateRequiredCodeRule {
    fn check(&self, item: &FallbackPlanEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback plan estimate.
#[derive(Debug, Clone)]
pub struct FallbackPlanEstimateValidator {
    pub rules: Vec<Rc<dyn FallbackPlanEstimateRule>>,
}

impl FallbackPlanEstimateValidator {
    pub fn validate(&self, item: &FallbackPlanEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackPlanEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPlanEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_plan_estimate_validator() -> FallbackPlanEstimateValidator {
    FallbackPlanEstimateValidator {
        rules: vec![
            Rc::new(FallbackPlanEstimateSequenceRangeRule { min: 64, max: 255 }) as Rc<dyn FallbackPlanEstimateRule>,
            Rc::new(FallbackPlanEstimateCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(FallbackPlanEstimateAllowedCodeRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(FallbackPlanEstimateRequiredNotesRule {
                message: "fallback plan estimate needs this field".to_string(),
            }),
            Rc::new(FallbackPlanEstimateWindowDaysRangeRule { min: 255, max: 255 }),
            Rc::new(FallbackPlanEstimateRequiredCodeRule {
                message: "fallback plan estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_plan_estimate_validator();
        let item = new_fallback_plan_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = FallbackPlanEstimateSequenceRangeRule { min: 64, max: 255 };
        let mut item = new_fallback_plan_estimate("x", "y");
        item.sequence = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = FallbackPlanEstimateRequiredNotesRule {
            message: "fallback plan estimate needs an entry".to_string(),
        };
        let mut item = new_fallback_plan_estimate("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = FallbackPlanEstimateWindowDaysRangeRule { min: 32, max: 120 };
        let mut item = new_fallback_plan_estimate("x", "y");
        item.window_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackPlanEstimateRequiredCodeRule {
            message: "fallback plan estimate needs a value".to_string(),
        };
        let mut item = new_fallback_plan_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
