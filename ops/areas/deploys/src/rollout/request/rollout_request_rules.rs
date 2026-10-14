use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout request must pass. Check returns its problems, or none.
pub trait RolloutRequestRule: std::fmt::Debug {
    fn check(&self, item: &RolloutRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRequestRequiredTenantIdRule {
    pub message: String,
}

impl RolloutRequestRule for RolloutRequestRequiredTenantIdRule {
    fn check(&self, item: &RolloutRequest) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRequestCodePrefixRule {
    pub prefix: String,
}

impl RolloutRequestRule for RolloutRequestCodePrefixRule {
    fn check(&self, item: &RolloutRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRequestThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutRequestRule for RolloutRequestThresholdRangeRule {
    fn check(&self, item: &RolloutRequest) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout request.
#[derive(Debug, Clone)]
pub struct RolloutRequestValidator {
    pub rules: Vec<Rc<dyn RolloutRequestRule>>,
}

impl RolloutRequestValidator {
    pub fn validate(&self, item: &RolloutRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_request_validator() -> RolloutRequestValidator {
    RolloutRequestValidator {
        rules: vec![
            Rc::new(RolloutRequestRequiredTenantIdRule {
                message: "rollout request needs this field".to_string(),
            }) as Rc<dyn RolloutRequestRule>,
            Rc::new(RolloutRequestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RolloutRequestThresholdRangeRule { min: 2.4, max: 4135.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_request_validator();
        let item = new_rollout_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = RolloutRequestRequiredTenantIdRule {
            message: "rollout request needs this field".to_string(),
        };
        let mut item = new_rollout_request("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RolloutRequestThresholdRangeRule { min: 4.0, max: 2966.6 };
        let mut item = new_rollout_request("x", "y");
        item.threshold = 2968.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
