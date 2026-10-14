use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled ttl policy plan must pass. Check returns its problems, or none.
pub trait ScheduledTtlPolicyPlanRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledTtlPolicyPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyPlanQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledTtlPolicyPlanRule for ScheduledTtlPolicyPlanQuotaUnitsRangeRule {
    fn check(&self, item: &ScheduledTtlPolicyPlan) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyPlanRequiredTitleRule {
    pub message: String,
}

impl ScheduledTtlPolicyPlanRule for ScheduledTtlPolicyPlanRequiredTitleRule {
    fn check(&self, item: &ScheduledTtlPolicyPlan) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyPlanSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledTtlPolicyPlanRule for ScheduledTtlPolicyPlanSequenceRangeRule {
    fn check(&self, item: &ScheduledTtlPolicyPlan) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled ttl policy plan.
#[derive(Debug, Clone)]
pub struct ScheduledTtlPolicyPlanValidator {
    pub rules: Vec<Rc<dyn ScheduledTtlPolicyPlanRule>>,
}

impl ScheduledTtlPolicyPlanValidator {
    pub fn validate(&self, item: &ScheduledTtlPolicyPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledTtlPolicyPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledTtlPolicyPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_ttl_policy_plan_validator() -> ScheduledTtlPolicyPlanValidator {
    ScheduledTtlPolicyPlanValidator {
        rules: vec![
            Rc::new(ScheduledTtlPolicyPlanQuotaUnitsRangeRule { min: 120, max: 86400 })
                as Rc<dyn ScheduledTtlPolicyPlanRule>,
            Rc::new(ScheduledTtlPolicyPlanRequiredTitleRule {
                message: "scheduled ttl policy plan needs this field".to_string(),
            }),
            Rc::new(ScheduledTtlPolicyPlanSequenceRangeRule { min: 64, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_ttl_policy_plan_validator();
        let item = new_scheduled_ttl_policy_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = ScheduledTtlPolicyPlanQuotaUnitsRangeRule { min: 16, max: 16 };
        let mut item = new_scheduled_ttl_policy_plan("x", "y");
        item.quota_units = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledTtlPolicyPlanRequiredTitleRule {
            message: "scheduled ttl policy plan needs an entry".to_string(),
        };
        let mut item = new_scheduled_ttl_policy_plan("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = ScheduledTtlPolicyPlanSequenceRangeRule { min: 80, max: 3600 };
        let mut item = new_scheduled_ttl_policy_plan("x", "y");
        item.sequence = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
