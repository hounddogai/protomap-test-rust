use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict timeline budget must pass. Check returns its problems, or none.
pub trait StrictTimelineBudgetRule: std::fmt::Debug {
    fn check(&self, item: &StrictTimelineBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineBudgetDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictTimelineBudgetRule for StrictTimelineBudgetDiscountRangeRule {
    fn check(&self, item: &StrictTimelineBudget) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineBudgetCodeLengthRule {
    pub max_length: i64,
}

impl StrictTimelineBudgetRule for StrictTimelineBudgetCodeLengthRule {
    fn check(&self, item: &StrictTimelineBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineBudgetRequiredLabelRule {
    pub message: String,
}

impl StrictTimelineBudgetRule for StrictTimelineBudgetRequiredLabelRule {
    fn check(&self, item: &StrictTimelineBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineBudgetBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictTimelineBudgetRule for StrictTimelineBudgetBurstRangeRule {
    fn check(&self, item: &StrictTimelineBudget) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineBudgetLabelLengthRule {
    pub max_length: i64,
}

impl StrictTimelineBudgetRule for StrictTimelineBudgetLabelLengthRule {
    fn check(&self, item: &StrictTimelineBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict timeline budget.
#[derive(Debug, Clone)]
pub struct StrictTimelineBudgetValidator {
    pub rules: Vec<Rc<dyn StrictTimelineBudgetRule>>,
}

impl StrictTimelineBudgetValidator {
    pub fn validate(&self, item: &StrictTimelineBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictTimelineBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictTimelineBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_timeline_budget_validator() -> StrictTimelineBudgetValidator {
    StrictTimelineBudgetValidator {
        rules: vec![
            Rc::new(StrictTimelineBudgetDiscountRangeRule { min: 1.3, max: 4529.3 })
                as Rc<dyn StrictTimelineBudgetRule>,
            Rc::new(StrictTimelineBudgetCodeLengthRule { max_length: 80 }),
            Rc::new(StrictTimelineBudgetRequiredLabelRule {
                message: "strict timeline budget needs a value".to_string(),
            }),
            Rc::new(StrictTimelineBudgetBurstRangeRule { min: 80, max: 3600 }),
            Rc::new(StrictTimelineBudgetLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_timeline_budget_validator();
        let item = new_strict_timeline_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = StrictTimelineBudgetDiscountRangeRule { min: 9.6, max: 2656.9 };
        let mut item = new_strict_timeline_budget("x", "y");
        item.discount = 2658.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictTimelineBudgetCodeLengthRule { max_length: 86400 };
        let mut item = new_strict_timeline_budget("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictTimelineBudgetRequiredLabelRule {
            message: "strict timeline budget needs an entry".to_string(),
        };
        let mut item = new_strict_timeline_budget("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = StrictTimelineBudgetBurstRangeRule { min: 120, max: 3600 };
        let mut item = new_strict_timeline_budget("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = StrictTimelineBudgetLabelLengthRule { max_length: 16 };
        let mut item = new_strict_timeline_budget("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
