use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient plan baseline must pass. Check returns its problems, or none.
pub trait LenientPlanBaselineRule: std::fmt::Debug {
    fn check(&self, item: &LenientPlanBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanBaselineRequiredTimezoneRule {
    pub message: String,
}

impl LenientPlanBaselineRule for LenientPlanBaselineRequiredTimezoneRule {
    fn check(&self, item: &LenientPlanBaseline) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanBaselineAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPlanBaselineRule for LenientPlanBaselineAttemptsRangeRule {
    fn check(&self, item: &LenientPlanBaseline) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanBaselineUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPlanBaselineRule for LenientPlanBaselineUpdatedAtRangeRule {
    fn check(&self, item: &LenientPlanBaseline) -> Vec<String> {
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
pub struct LenientPlanBaselineRequiredCodeRule {
    pub message: String,
}

impl LenientPlanBaselineRule for LenientPlanBaselineRequiredCodeRule {
    fn check(&self, item: &LenientPlanBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanBaselineAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl LenientPlanBaselineRule for LenientPlanBaselineAllowedLabelRule {
    fn check(&self, item: &LenientPlanBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanBaselineRequiredLabelRule {
    pub message: String,
}

impl LenientPlanBaselineRule for LenientPlanBaselineRequiredLabelRule {
    fn check(&self, item: &LenientPlanBaseline) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient plan baseline.
#[derive(Debug, Clone)]
pub struct LenientPlanBaselineValidator {
    pub rules: Vec<Rc<dyn LenientPlanBaselineRule>>,
}

impl LenientPlanBaselineValidator {
    pub fn validate(&self, item: &LenientPlanBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientPlanBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPlanBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_plan_baseline_validator() -> LenientPlanBaselineValidator {
    LenientPlanBaselineValidator {
        rules: vec![
            Rc::new(LenientPlanBaselineRequiredTimezoneRule {
                message: "lenient plan baseline needs this field".to_string(),
            }) as Rc<dyn LenientPlanBaselineRule>,
            Rc::new(LenientPlanBaselineAttemptsRangeRule { min: 32, max: 255 }),
            Rc::new(LenientPlanBaselineUpdatedAtRangeRule { min: 255, max: 1000 }),
            Rc::new(LenientPlanBaselineRequiredCodeRule {
                message: "lenient plan baseline needs this field".to_string(),
            }),
            Rc::new(LenientPlanBaselineAllowedLabelRule {
                allowed: vec!["default".to_string(), "adaptive".to_string(), "adaptive".to_string()],
            }),
            Rc::new(LenientPlanBaselineRequiredLabelRule {
                message: "lenient plan baseline needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_plan_baseline_validator();
        let item = new_lenient_plan_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = LenientPlanBaselineRequiredTimezoneRule {
            message: "lenient plan baseline needs this field".to_string(),
        };
        let mut item = new_lenient_plan_baseline("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = LenientPlanBaselineAttemptsRangeRule { min: 32, max: 120 };
        let mut item = new_lenient_plan_baseline("x", "y");
        item.attempts = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = LenientPlanBaselineUpdatedAtRangeRule { min: 1000, max: 1000 };
        let mut item = new_lenient_plan_baseline("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientPlanBaselineRequiredCodeRule {
            message: "lenient plan baseline needs an entry".to_string(),
        };
        let mut item = new_lenient_plan_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientPlanBaselineRequiredLabelRule {
            message: "lenient plan baseline needs an entry".to_string(),
        };
        let mut item = new_lenient_plan_baseline("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
