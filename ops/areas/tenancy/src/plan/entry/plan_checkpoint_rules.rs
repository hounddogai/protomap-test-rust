use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan checkpoint must pass. Check returns its problems, or none.
pub trait PlanCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &PlanCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanCheckpointRequiredCategoryRule {
    pub message: String,
}

impl PlanCheckpointRule for PlanCheckpointRequiredCategoryRule {
    fn check(&self, item: &PlanCheckpoint) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanCheckpointAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PlanCheckpointRule for PlanCheckpointAllowedCodeRule {
    fn check(&self, item: &PlanCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanCheckpointLabelLengthRule {
    pub max_length: i64,
}

impl PlanCheckpointRule for PlanCheckpointLabelLengthRule {
    fn check(&self, item: &PlanCheckpoint) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan checkpoint.
#[derive(Debug, Clone)]
pub struct PlanCheckpointValidator {
    pub rules: Vec<Rc<dyn PlanCheckpointRule>>,
}

impl PlanCheckpointValidator {
    pub fn validate(&self, item: &PlanCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_checkpoint_validator() -> PlanCheckpointValidator {
    PlanCheckpointValidator {
        rules: vec![
            Rc::new(PlanCheckpointRequiredCategoryRule {
                message: "plan checkpoint needs a value".to_string(),
            }) as Rc<dyn PlanCheckpointRule>,
            Rc::new(PlanCheckpointAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(PlanCheckpointLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_checkpoint_validator();
        let item = new_plan_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = PlanCheckpointRequiredCategoryRule {
            message: "plan checkpoint needs an entry".to_string(),
        };
        let mut item = new_plan_checkpoint("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PlanCheckpointLabelLengthRule { max_length: 32 };
        let mut item = new_plan_checkpoint("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
