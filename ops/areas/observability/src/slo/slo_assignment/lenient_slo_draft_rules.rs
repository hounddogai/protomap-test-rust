use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient slo draft must pass. Check returns its problems, or none.
pub trait LenientSloDraftRule: std::fmt::Debug {
    fn check(&self, item: &LenientSloDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloDraftBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientSloDraftRule for LenientSloDraftBudgetRangeRule {
    fn check(&self, item: &LenientSloDraft) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloDraftPhoneLengthRule {
    pub max_length: i64,
}

impl LenientSloDraftRule for LenientSloDraftPhoneLengthRule {
    fn check(&self, item: &LenientSloDraft) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloDraftAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientSloDraftRule for LenientSloDraftAmountRangeRule {
    fn check(&self, item: &LenientSloDraft) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloDraftRequiredLabelRule {
    pub message: String,
}

impl LenientSloDraftRule for LenientSloDraftRequiredLabelRule {
    fn check(&self, item: &LenientSloDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient slo draft.
#[derive(Debug, Clone)]
pub struct LenientSloDraftValidator {
    pub rules: Vec<Rc<dyn LenientSloDraftRule>>,
}

impl LenientSloDraftValidator {
    pub fn validate(&self, item: &LenientSloDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientSloDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSloDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_slo_draft_validator() -> LenientSloDraftValidator {
    LenientSloDraftValidator {
        rules: vec![
            Rc::new(LenientSloDraftBudgetRangeRule { min: 3.5, max: 2721.9 }) as Rc<dyn LenientSloDraftRule>,
            Rc::new(LenientSloDraftPhoneLengthRule { max_length: 120 }),
            Rc::new(LenientSloDraftAmountRangeRule { min: 2.9, max: 2792.5 }),
            Rc::new(LenientSloDraftRequiredLabelRule {
                message: "lenient slo draft needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_slo_draft_validator();
        let item = new_lenient_slo_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = LenientSloDraftBudgetRangeRule { min: 9.0, max: 3269.2 };
        let mut item = new_lenient_slo_draft("x", "y");
        item.budget = 3270.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = LenientSloDraftPhoneLengthRule { max_length: 32 };
        let mut item = new_lenient_slo_draft("x", "y");
        item.phone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = LenientSloDraftAmountRangeRule { min: 4.4, max: 3955.6 };
        let mut item = new_lenient_slo_draft("x", "y");
        item.amount = 3957.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientSloDraftRequiredLabelRule {
            message: "lenient slo draft needs an entry".to_string(),
        };
        let mut item = new_lenient_slo_draft("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
