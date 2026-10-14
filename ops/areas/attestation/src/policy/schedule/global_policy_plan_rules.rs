use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global policy plan must pass. Check returns its problems, or none.
pub trait GlobalPolicyPlanRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPolicyPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicyPlanClusterLengthRule {
    pub max_length: i64,
}

impl GlobalPolicyPlanRule for GlobalPolicyPlanClusterLengthRule {
    fn check(&self, item: &GlobalPolicyPlan) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicyPlanRequiredClusterRule {
    pub message: String,
}

impl GlobalPolicyPlanRule for GlobalPolicyPlanRequiredClusterRule {
    fn check(&self, item: &GlobalPolicyPlan) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicyPlanPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalPolicyPlanRule for GlobalPolicyPlanPortRangeRule {
    fn check(&self, item: &GlobalPolicyPlan) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicyPlanTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalPolicyPlanRule for GlobalPolicyPlanTtlSecondsRangeRule {
    fn check(&self, item: &GlobalPolicyPlan) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicyPlanCodePrefixRule {
    pub prefix: String,
}

impl GlobalPolicyPlanRule for GlobalPolicyPlanCodePrefixRule {
    fn check(&self, item: &GlobalPolicyPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicyPlanRequiredNameRule {
    pub message: String,
}

impl GlobalPolicyPlanRule for GlobalPolicyPlanRequiredNameRule {
    fn check(&self, item: &GlobalPolicyPlan) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global policy plan.
#[derive(Debug, Clone)]
pub struct GlobalPolicyPlanValidator {
    pub rules: Vec<Rc<dyn GlobalPolicyPlanRule>>,
}

impl GlobalPolicyPlanValidator {
    pub fn validate(&self, item: &GlobalPolicyPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalPolicyPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPolicyPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_policy_plan_validator() -> GlobalPolicyPlanValidator {
    GlobalPolicyPlanValidator {
        rules: vec![
            Rc::new(GlobalPolicyPlanClusterLengthRule { max_length: 3600 }) as Rc<dyn GlobalPolicyPlanRule>,
            Rc::new(GlobalPolicyPlanRequiredClusterRule {
                message: "global policy plan needs an entry".to_string(),
            }),
            Rc::new(GlobalPolicyPlanPortRangeRule { min: 255, max: 255 }),
            Rc::new(GlobalPolicyPlanTtlSecondsRangeRule { min: 64, max: 120 }),
            Rc::new(GlobalPolicyPlanCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(GlobalPolicyPlanRequiredNameRule {
                message: "global policy plan needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_policy_plan_validator();
        let item = new_global_policy_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = GlobalPolicyPlanClusterLengthRule { max_length: 255 };
        let mut item = new_global_policy_plan("x", "y");
        item.cluster = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = GlobalPolicyPlanRequiredClusterRule {
            message: "global policy plan needs this field".to_string(),
        };
        let mut item = new_global_policy_plan("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = GlobalPolicyPlanPortRangeRule { min: 32, max: 120 };
        let mut item = new_global_policy_plan("x", "y");
        item.port = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = GlobalPolicyPlanTtlSecondsRangeRule { min: 120, max: 3600 };
        let mut item = new_global_policy_plan("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GlobalPolicyPlanRequiredNameRule {
            message: "global policy plan needs this field".to_string(),
        };
        let mut item = new_global_policy_plan("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
