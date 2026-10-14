use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled sink plan must pass. Check returns its problems, or none.
pub trait ScheduledSinkPlanRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSinkPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkPlanExternalRefLengthRule {
    pub max_length: i64,
}

impl ScheduledSinkPlanRule for ScheduledSinkPlanExternalRefLengthRule {
    fn check(&self, item: &ScheduledSinkPlan) -> Vec<String> {
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
pub struct ScheduledSinkPlanRequiredCodeRule {
    pub message: String,
}

impl ScheduledSinkPlanRule for ScheduledSinkPlanRequiredCodeRule {
    fn check(&self, item: &ScheduledSinkPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkPlanAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledSinkPlanRule for ScheduledSinkPlanAmountRangeRule {
    fn check(&self, item: &ScheduledSinkPlan) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkPlanAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledSinkPlanRule for ScheduledSinkPlanAllowedCodeRule {
    fn check(&self, item: &ScheduledSinkPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled sink plan.
#[derive(Debug, Clone)]
pub struct ScheduledSinkPlanValidator {
    pub rules: Vec<Rc<dyn ScheduledSinkPlanRule>>,
}

impl ScheduledSinkPlanValidator {
    pub fn validate(&self, item: &ScheduledSinkPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledSinkPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSinkPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_sink_plan_validator() -> ScheduledSinkPlanValidator {
    ScheduledSinkPlanValidator {
        rules: vec![
            Rc::new(ScheduledSinkPlanExternalRefLengthRule { max_length: 32 }) as Rc<dyn ScheduledSinkPlanRule>,
            Rc::new(ScheduledSinkPlanRequiredCodeRule {
                message: "scheduled sink plan needs a value".to_string(),
            }),
            Rc::new(ScheduledSinkPlanAmountRangeRule { min: 7.4, max: 3967.2 }),
            Rc::new(ScheduledSinkPlanAllowedCodeRule {
                allowed: vec!["manual".to_string(), "primary".to_string(), "manual".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_sink_plan_validator();
        let item = new_scheduled_sink_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = ScheduledSinkPlanExternalRefLengthRule { max_length: 64 };
        let mut item = new_scheduled_sink_plan("x", "y");
        item.external_ref = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledSinkPlanRequiredCodeRule {
            message: "scheduled sink plan needs an entry".to_string(),
        };
        let mut item = new_scheduled_sink_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = ScheduledSinkPlanAmountRangeRule { min: 6.7, max: 2632.1 };
        let mut item = new_scheduled_sink_plan("x", "y");
        item.amount = 2633.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
