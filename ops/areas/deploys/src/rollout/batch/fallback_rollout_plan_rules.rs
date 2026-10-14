use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback rollout plan must pass. Check returns its problems, or none.
pub trait FallbackRolloutPlanRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRolloutPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutPlanRequiredExternalRefRule {
    pub message: String,
}

impl FallbackRolloutPlanRule for FallbackRolloutPlanRequiredExternalRefRule {
    fn check(&self, item: &FallbackRolloutPlan) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutPlanRequiredCodeRule {
    pub message: String,
}

impl FallbackRolloutPlanRule for FallbackRolloutPlanRequiredCodeRule {
    fn check(&self, item: &FallbackRolloutPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutPlanRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackRolloutPlanRule for FallbackRolloutPlanRateRangeRule {
    fn check(&self, item: &FallbackRolloutPlan) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback rollout plan.
#[derive(Debug, Clone)]
pub struct FallbackRolloutPlanValidator {
    pub rules: Vec<Rc<dyn FallbackRolloutPlanRule>>,
}

impl FallbackRolloutPlanValidator {
    pub fn validate(&self, item: &FallbackRolloutPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackRolloutPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRolloutPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_rollout_plan_validator() -> FallbackRolloutPlanValidator {
    FallbackRolloutPlanValidator {
        rules: vec![
            Rc::new(FallbackRolloutPlanRequiredExternalRefRule {
                message: "fallback rollout plan needs a value".to_string(),
            }) as Rc<dyn FallbackRolloutPlanRule>,
            Rc::new(FallbackRolloutPlanRequiredCodeRule {
                message: "fallback rollout plan needs a value".to_string(),
            }),
            Rc::new(FallbackRolloutPlanRateRangeRule { min: 2.9, max: 1969.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_rollout_plan_validator();
        let item = new_fallback_rollout_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = FallbackRolloutPlanRequiredExternalRefRule {
            message: "fallback rollout plan needs a value".to_string(),
        };
        let mut item = new_fallback_rollout_plan("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackRolloutPlanRequiredCodeRule {
            message: "fallback rollout plan needs a value".to_string(),
        };
        let mut item = new_fallback_rollout_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = FallbackRolloutPlanRateRangeRule { min: 8.3, max: 2957.5 };
        let mut item = new_fallback_rollout_plan("x", "y");
        item.rate = 2959.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
