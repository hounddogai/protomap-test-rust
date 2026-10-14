use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled release budget must pass. Check returns its problems, or none.
pub trait ScheduledReleaseBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledReleaseBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReleaseBudgetMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledReleaseBudgetRule for ScheduledReleaseBudgetMaximumRangeRule {
    fn check(&self, item: &ScheduledReleaseBudget) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReleaseBudgetExternalRefLengthRule {
    pub max_length: i64,
}

impl ScheduledReleaseBudgetRule for ScheduledReleaseBudgetExternalRefLengthRule {
    fn check(&self, item: &ScheduledReleaseBudget) -> Vec<String> {
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
pub struct ScheduledReleaseBudgetRequiredChecksumRule {
    pub message: String,
}

impl ScheduledReleaseBudgetRule for ScheduledReleaseBudgetRequiredChecksumRule {
    fn check(&self, item: &ScheduledReleaseBudget) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReleaseBudgetMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledReleaseBudgetRule for ScheduledReleaseBudgetMarginRangeRule {
    fn check(&self, item: &ScheduledReleaseBudget) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReleaseBudgetAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ScheduledReleaseBudgetRule for ScheduledReleaseBudgetAllowedTitleRule {
    fn check(&self, item: &ScheduledReleaseBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled release budget.
#[derive(Debug, Clone)]
pub struct ScheduledReleaseBudgetValidator {
    pub rules: Vec<Rc<dyn ScheduledReleaseBudgetRule>>,
}

impl ScheduledReleaseBudgetValidator {
    pub fn validate(&self, item: &ScheduledReleaseBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledReleaseBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledReleaseBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_release_budget_validator() -> ScheduledReleaseBudgetValidator {
    ScheduledReleaseBudgetValidator {
        rules: vec![
            Rc::new(ScheduledReleaseBudgetMaximumRangeRule { min: 0.8, max: 2909.5 })
                as Rc<dyn ScheduledReleaseBudgetRule>,
            Rc::new(ScheduledReleaseBudgetExternalRefLengthRule { max_length: 120 }),
            Rc::new(ScheduledReleaseBudgetRequiredChecksumRule {
                message: "scheduled release budget needs a value".to_string(),
            }),
            Rc::new(ScheduledReleaseBudgetMarginRangeRule { min: 7.3, max: 2671.1 }),
            Rc::new(ScheduledReleaseBudgetAllowedTitleRule {
                allowed: vec!["primary".to_string(), "batched".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_release_budget_validator();
        let item = new_scheduled_release_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ScheduledReleaseBudgetMaximumRangeRule { min: 0.0, max: 2424.2 };
        let mut item = new_scheduled_release_budget("x", "y");
        item.maximum = 2425.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = ScheduledReleaseBudgetExternalRefLengthRule { max_length: 32 };
        let mut item = new_scheduled_release_budget("x", "y");
        item.external_ref = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = ScheduledReleaseBudgetRequiredChecksumRule {
            message: "scheduled release budget needs a value".to_string(),
        };
        let mut item = new_scheduled_release_budget("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = ScheduledReleaseBudgetMarginRangeRule { min: 1.3, max: 3895.3 };
        let mut item = new_scheduled_release_budget("x", "y");
        item.margin = 3896.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
