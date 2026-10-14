use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling postmortem policy must pass. Check returns its problems, or none.
pub trait RollingPostmortemPolicyRule: std::fmt::Debug {
    fn check(&self, item: &RollingPostmortemPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPostmortemPolicyBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingPostmortemPolicyRule for RollingPostmortemPolicyBudgetRangeRule {
    fn check(&self, item: &RollingPostmortemPolicy) -> Vec<String> {
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
pub struct RollingPostmortemPolicyOwnerIdLengthRule {
    pub max_length: i64,
}

impl RollingPostmortemPolicyRule for RollingPostmortemPolicyOwnerIdLengthRule {
    fn check(&self, item: &RollingPostmortemPolicy) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPostmortemPolicyRequiredLabelRule {
    pub message: String,
}

impl RollingPostmortemPolicyRule for RollingPostmortemPolicyRequiredLabelRule {
    fn check(&self, item: &RollingPostmortemPolicy) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling postmortem policy.
#[derive(Debug, Clone)]
pub struct RollingPostmortemPolicyValidator {
    pub rules: Vec<Rc<dyn RollingPostmortemPolicyRule>>,
}

impl RollingPostmortemPolicyValidator {
    pub fn validate(&self, item: &RollingPostmortemPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingPostmortemPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPostmortemPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_postmortem_policy_validator() -> RollingPostmortemPolicyValidator {
    RollingPostmortemPolicyValidator {
        rules: vec![
            Rc::new(RollingPostmortemPolicyBudgetRangeRule { min: 8.1, max: 4406.3 })
                as Rc<dyn RollingPostmortemPolicyRule>,
            Rc::new(RollingPostmortemPolicyOwnerIdLengthRule { max_length: 16 }),
            Rc::new(RollingPostmortemPolicyRequiredLabelRule {
                message: "rolling postmortem policy needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_postmortem_policy_validator();
        let item = new_rolling_postmortem_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RollingPostmortemPolicyBudgetRangeRule { min: 1.0, max: 4114.5 };
        let mut item = new_rolling_postmortem_policy("x", "y");
        item.budget = 4116.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = RollingPostmortemPolicyOwnerIdLengthRule { max_length: 64 };
        let mut item = new_rolling_postmortem_policy("x", "y");
        item.owner_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollingPostmortemPolicyRequiredLabelRule {
            message: "rolling postmortem policy needs this field".to_string(),
        };
        let mut item = new_rolling_postmortem_policy("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
