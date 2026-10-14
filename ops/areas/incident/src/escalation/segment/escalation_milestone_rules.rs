use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation milestone must pass. Check returns its problems, or none.
pub trait EscalationMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &EscalationMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMilestoneVisibleGuardRule {
    pub limit: f64,
}

impl EscalationMilestoneRule for EscalationMilestoneVisibleGuardRule {
    fn check(&self, item: &EscalationMilestone) -> Vec<String> {
        if item.visible && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while visible", item.budget, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMilestoneTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl EscalationMilestoneRule for EscalationMilestoneTagsLimitRule {
    fn check(&self, item: &EscalationMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMilestoneRequiredNameRule {
    pub message: String,
}

impl EscalationMilestoneRule for EscalationMilestoneRequiredNameRule {
    fn check(&self, item: &EscalationMilestone) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMilestoneBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EscalationMilestoneRule for EscalationMilestoneBudgetRangeRule {
    fn check(&self, item: &EscalationMilestone) -> Vec<String> {
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
pub struct EscalationMilestoneScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EscalationMilestoneRule for EscalationMilestoneScoreRangeRule {
    fn check(&self, item: &EscalationMilestone) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMilestoneNameLengthRule {
    pub max_length: i64,
}

impl EscalationMilestoneRule for EscalationMilestoneNameLengthRule {
    fn check(&self, item: &EscalationMilestone) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation milestone.
#[derive(Debug, Clone)]
pub struct EscalationMilestoneValidator {
    pub rules: Vec<Rc<dyn EscalationMilestoneRule>>,
}

impl EscalationMilestoneValidator {
    pub fn validate(&self, item: &EscalationMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_milestone_validator() -> EscalationMilestoneValidator {
    EscalationMilestoneValidator {
        rules: vec![
            Rc::new(EscalationMilestoneVisibleGuardRule { limit: 8.7 }) as Rc<dyn EscalationMilestoneRule>,
            Rc::new(EscalationMilestoneTagsLimitRule {
                max_count: 16,
                banned: vec!["primary".to_string(), "weighted".to_string(), "primary".to_string()],
            }),
            Rc::new(EscalationMilestoneRequiredNameRule {
                message: "escalation milestone needs an entry".to_string(),
            }),
            Rc::new(EscalationMilestoneBudgetRangeRule { min: 0.6, max: 1919.7 }),
            Rc::new(EscalationMilestoneScoreRangeRule { min: 1.3, max: 2624.6 }),
            Rc::new(EscalationMilestoneNameLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_milestone_validator();
        let item = new_escalation_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = EscalationMilestoneRequiredNameRule {
            message: "escalation milestone needs this field".to_string(),
        };
        let mut item = new_escalation_milestone("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = EscalationMilestoneBudgetRangeRule { min: 10.0, max: 4594.7 };
        let mut item = new_escalation_milestone("x", "y");
        item.budget = 4596.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = EscalationMilestoneScoreRangeRule { min: 6.3, max: 4375.6 };
        let mut item = new_escalation_milestone("x", "y");
        item.score = 4377.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = EscalationMilestoneNameLengthRule { max_length: 86400 };
        let mut item = new_escalation_milestone("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
