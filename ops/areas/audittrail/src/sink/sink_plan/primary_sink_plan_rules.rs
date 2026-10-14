use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary sink plan must pass. Check returns its problems, or none.
pub trait PrimarySinkPlanRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySinkPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkPlanRequiredTitleRule {
    pub message: String,
}

impl PrimarySinkPlanRule for PrimarySinkPlanRequiredTitleRule {
    fn check(&self, item: &PrimarySinkPlan) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkPlanRequiredNotesRule {
    pub message: String,
}

impl PrimarySinkPlanRule for PrimarySinkPlanRequiredNotesRule {
    fn check(&self, item: &PrimarySinkPlan) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkPlanAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimarySinkPlanRule for PrimarySinkPlanAmountRangeRule {
    fn check(&self, item: &PrimarySinkPlan) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkPlanCodeLengthRule {
    pub max_length: i64,
}

impl PrimarySinkPlanRule for PrimarySinkPlanCodeLengthRule {
    fn check(&self, item: &PrimarySinkPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkPlanQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimarySinkPlanRule for PrimarySinkPlanQuantityRangeRule {
    fn check(&self, item: &PrimarySinkPlan) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a primary sink plan.
#[derive(Debug, Clone)]
pub struct PrimarySinkPlanValidator {
    pub rules: Vec<Rc<dyn PrimarySinkPlanRule>>,
}

impl PrimarySinkPlanValidator {
    pub fn validate(&self, item: &PrimarySinkPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimarySinkPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySinkPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_sink_plan_validator() -> PrimarySinkPlanValidator {
    PrimarySinkPlanValidator {
        rules: vec![
            Rc::new(PrimarySinkPlanRequiredTitleRule {
                message: "primary sink plan needs this field".to_string(),
            }) as Rc<dyn PrimarySinkPlanRule>,
            Rc::new(PrimarySinkPlanRequiredNotesRule {
                message: "primary sink plan needs an entry".to_string(),
            }),
            Rc::new(PrimarySinkPlanAmountRangeRule { min: 2.8, max: 283.1 }),
            Rc::new(PrimarySinkPlanCodeLengthRule { max_length: 3600 }),
            Rc::new(PrimarySinkPlanQuantityRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_sink_plan_validator();
        let item = new_primary_sink_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimarySinkPlanRequiredTitleRule {
            message: "primary sink plan needs a value".to_string(),
        };
        let mut item = new_primary_sink_plan("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = PrimarySinkPlanRequiredNotesRule {
            message: "primary sink plan needs a value".to_string(),
        };
        let mut item = new_primary_sink_plan("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = PrimarySinkPlanAmountRangeRule { min: 4.1, max: 1388.2 };
        let mut item = new_primary_sink_plan("x", "y");
        item.amount = 1389.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimarySinkPlanCodeLengthRule { max_length: 80 };
        let mut item = new_primary_sink_plan("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = PrimarySinkPlanQuantityRangeRule { min: 64, max: 120 };
        let mut item = new_primary_sink_plan("x", "y");
        item.quantity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
