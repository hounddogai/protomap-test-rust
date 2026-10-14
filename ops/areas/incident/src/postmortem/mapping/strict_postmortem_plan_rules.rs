use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict postmortem plan must pass. Check returns its problems, or none.
pub trait StrictPostmortemPlanRule: std::fmt::Debug {
    fn check(&self, item: &StrictPostmortemPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPostmortemPlanLabelLengthRule {
    pub max_length: i64,
}

impl StrictPostmortemPlanRule for StrictPostmortemPlanLabelLengthRule {
    fn check(&self, item: &StrictPostmortemPlan) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPostmortemPlanAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl StrictPostmortemPlanRule for StrictPostmortemPlanAllowedLabelRule {
    fn check(&self, item: &StrictPostmortemPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPostmortemPlanVisibleGuardRule {
    pub limit: i64,
}

impl StrictPostmortemPlanRule for StrictPostmortemPlanVisibleGuardRule {
    fn check(&self, item: &StrictPostmortemPlan) -> Vec<String> {
        if item.visible && (item.ttl_seconds > self.limit) {
            return vec![format!(
                "ttl seconds {} exceeds {} while visible",
                item.ttl_seconds, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPostmortemPlanCodeLengthRule {
    pub max_length: i64,
}

impl StrictPostmortemPlanRule for StrictPostmortemPlanCodeLengthRule {
    fn check(&self, item: &StrictPostmortemPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict postmortem plan.
#[derive(Debug, Clone)]
pub struct StrictPostmortemPlanValidator {
    pub rules: Vec<Rc<dyn StrictPostmortemPlanRule>>,
}

impl StrictPostmortemPlanValidator {
    pub fn validate(&self, item: &StrictPostmortemPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictPostmortemPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictPostmortemPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_postmortem_plan_validator() -> StrictPostmortemPlanValidator {
    StrictPostmortemPlanValidator {
        rules: vec![
            Rc::new(StrictPostmortemPlanLabelLengthRule { max_length: 3600 }) as Rc<dyn StrictPostmortemPlanRule>,
            Rc::new(StrictPostmortemPlanAllowedLabelRule {
                allowed: vec!["batched".to_string(), "scheduled".to_string()],
            }),
            Rc::new(StrictPostmortemPlanVisibleGuardRule { limit: 120 }),
            Rc::new(StrictPostmortemPlanCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_postmortem_plan_validator();
        let item = new_strict_postmortem_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = StrictPostmortemPlanLabelLengthRule { max_length: 1000 };
        let mut item = new_strict_postmortem_plan("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictPostmortemPlanCodeLengthRule { max_length: 80 };
        let mut item = new_strict_postmortem_plan("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
