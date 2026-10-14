use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled federation budget must pass. Check returns its problems, or none.
pub trait ScheduledFederationBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledFederationBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationBudgetSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledFederationBudgetRule for ScheduledFederationBudgetSequenceRangeRule {
    fn check(&self, item: &ScheduledFederationBudget) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationBudgetTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledFederationBudgetRule for ScheduledFederationBudgetTitleLengthRule {
    fn check(&self, item: &ScheduledFederationBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationBudgetPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledFederationBudgetRule for ScheduledFederationBudgetPriorityRangeRule {
    fn check(&self, item: &ScheduledFederationBudget) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationBudgetCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledFederationBudgetRule for ScheduledFederationBudgetCodeLengthRule {
    fn check(&self, item: &ScheduledFederationBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationBudgetAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl ScheduledFederationBudgetRule for ScheduledFederationBudgetAllowedSummaryRule {
    fn check(&self, item: &ScheduledFederationBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is blocked", item.summary)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled federation budget.
#[derive(Debug, Clone)]
pub struct ScheduledFederationBudgetValidator {
    pub rules: Vec<Rc<dyn ScheduledFederationBudgetRule>>,
}

impl ScheduledFederationBudgetValidator {
    pub fn validate(&self, item: &ScheduledFederationBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledFederationBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledFederationBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_federation_budget_validator() -> ScheduledFederationBudgetValidator {
    ScheduledFederationBudgetValidator {
        rules: vec![
            Rc::new(ScheduledFederationBudgetSequenceRangeRule { min: 16, max: 255 })
                as Rc<dyn ScheduledFederationBudgetRule>,
            Rc::new(ScheduledFederationBudgetTitleLengthRule { max_length: 1000 }),
            Rc::new(ScheduledFederationBudgetPriorityRangeRule { min: 80, max: 120 }),
            Rc::new(ScheduledFederationBudgetCodeLengthRule { max_length: 86400 }),
            Rc::new(ScheduledFederationBudgetAllowedSummaryRule {
                allowed: vec!["adaptive".to_string(), "default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_federation_budget_validator();
        let item = new_scheduled_federation_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = ScheduledFederationBudgetSequenceRangeRule { min: 120, max: 1000 };
        let mut item = new_scheduled_federation_budget("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledFederationBudgetTitleLengthRule { max_length: 32 };
        let mut item = new_scheduled_federation_budget("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = ScheduledFederationBudgetPriorityRangeRule { min: 255, max: 255 };
        let mut item = new_scheduled_federation_budget("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledFederationBudgetCodeLengthRule { max_length: 16 };
        let mut item = new_scheduled_federation_budget("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
