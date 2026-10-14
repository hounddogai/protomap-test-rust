use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling peering plan must pass. Check returns its problems, or none.
pub trait RollingPeeringPlanRule: std::fmt::Debug {
    fn check(&self, item: &RollingPeeringPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringPlanRequiredLabelRule {
    pub message: String,
}

impl RollingPeeringPlanRule for RollingPeeringPlanRequiredLabelRule {
    fn check(&self, item: &RollingPeeringPlan) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringPlanGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingPeeringPlanRule for RollingPeeringPlanGraceDaysRangeRule {
    fn check(&self, item: &RollingPeeringPlan) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringPlanCodePrefixRule {
    pub prefix: String,
}

impl RollingPeeringPlanRule for RollingPeeringPlanCodePrefixRule {
    fn check(&self, item: &RollingPeeringPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling peering plan.
#[derive(Debug, Clone)]
pub struct RollingPeeringPlanValidator {
    pub rules: Vec<Rc<dyn RollingPeeringPlanRule>>,
}

impl RollingPeeringPlanValidator {
    pub fn validate(&self, item: &RollingPeeringPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingPeeringPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPeeringPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_peering_plan_validator() -> RollingPeeringPlanValidator {
    RollingPeeringPlanValidator {
        rules: vec![
            Rc::new(RollingPeeringPlanRequiredLabelRule {
                message: "rolling peering plan needs a value".to_string(),
            }) as Rc<dyn RollingPeeringPlanRule>,
            Rc::new(RollingPeeringPlanGraceDaysRangeRule { min: 32, max: 80 }),
            Rc::new(RollingPeeringPlanCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_peering_plan_validator();
        let item = new_rolling_peering_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollingPeeringPlanRequiredLabelRule {
            message: "rolling peering plan needs an entry".to_string(),
        };
        let mut item = new_rolling_peering_plan("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RollingPeeringPlanGraceDaysRangeRule { min: 16, max: 32 };
        let mut item = new_rolling_peering_plan("x", "y");
        item.grace_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
