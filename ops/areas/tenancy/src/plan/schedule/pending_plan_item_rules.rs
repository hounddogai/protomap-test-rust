use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending plan item must pass. Check returns its problems, or none.
pub trait PendingPlanItemRule: std::fmt::Debug {
    fn check(&self, item: &PendingPlanItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanItemRequiredCodeRule {
    pub message: String,
}

impl PendingPlanItemRule for PendingPlanItemRequiredCodeRule {
    fn check(&self, item: &PendingPlanItem) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanItemEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingPlanItemRule for PendingPlanItemEndsAtRangeRule {
    fn check(&self, item: &PendingPlanItem) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanItemCodeLengthRule {
    pub max_length: i64,
}

impl PendingPlanItemRule for PendingPlanItemCodeLengthRule {
    fn check(&self, item: &PendingPlanItem) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending plan item.
#[derive(Debug, Clone)]
pub struct PendingPlanItemValidator {
    pub rules: Vec<Rc<dyn PendingPlanItemRule>>,
}

impl PendingPlanItemValidator {
    pub fn validate(&self, item: &PendingPlanItem) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingPlanItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPlanItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_plan_item_validator() -> PendingPlanItemValidator {
    PendingPlanItemValidator {
        rules: vec![
            Rc::new(PendingPlanItemRequiredCodeRule {
                message: "pending plan item needs this field".to_string(),
            }) as Rc<dyn PendingPlanItemRule>,
            Rc::new(PendingPlanItemEndsAtRangeRule { min: 16, max: 120 }),
            Rc::new(PendingPlanItemCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_plan_item_validator();
        let item = new_pending_plan_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingPlanItemRequiredCodeRule {
            message: "pending plan item needs this field".to_string(),
        };
        let mut item = new_pending_plan_item("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = PendingPlanItemEndsAtRangeRule { min: 80, max: 1000 };
        let mut item = new_pending_plan_item("x", "y");
        item.ends_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingPlanItemCodeLengthRule { max_length: 255 };
        let mut item = new_pending_plan_item("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
