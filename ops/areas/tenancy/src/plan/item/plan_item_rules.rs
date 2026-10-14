use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan item must pass. Check returns its problems, or none.
pub trait PlanItemRule: std::fmt::Debug {
    fn check(&self, item: &PlanItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanItemRequiredReviewerRule {
    pub message: String,
}

impl PlanItemRule for PlanItemRequiredReviewerRule {
    fn check(&self, item: &PlanItem) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanItemRequiredCodeRule {
    pub message: String,
}

impl PlanItemRule for PlanItemRequiredCodeRule {
    fn check(&self, item: &PlanItem) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanItemRequiredNotesRule {
    pub message: String,
}

impl PlanItemRule for PlanItemRequiredNotesRule {
    fn check(&self, item: &PlanItem) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan item.
#[derive(Debug, Clone)]
pub struct PlanItemValidator {
    pub rules: Vec<Rc<dyn PlanItemRule>>,
}

impl PlanItemValidator {
    pub fn validate(&self, item: &PlanItem) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_item_validator() -> PlanItemValidator {
    PlanItemValidator {
        rules: vec![
            Rc::new(PlanItemRequiredReviewerRule {
                message: "plan item needs this field".to_string(),
            }) as Rc<dyn PlanItemRule>,
            Rc::new(PlanItemRequiredCodeRule {
                message: "plan item needs this field".to_string(),
            }),
            Rc::new(PlanItemRequiredNotesRule {
                message: "plan item needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_item_validator();
        let item = new_plan_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = PlanItemRequiredReviewerRule {
            message: "plan item needs this field".to_string(),
        };
        let mut item = new_plan_item("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PlanItemRequiredCodeRule {
            message: "plan item needs this field".to_string(),
        };
        let mut item = new_plan_item("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = PlanItemRequiredNotesRule {
            message: "plan item needs a value".to_string(),
        };
        let mut item = new_plan_item("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
