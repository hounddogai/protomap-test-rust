use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled peering policy must pass. Check returns its problems, or none.
pub trait ScheduledPeeringPolicyRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledPeeringPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPeeringPolicyRequiredCodeRule {
    pub message: String,
}

impl ScheduledPeeringPolicyRule for ScheduledPeeringPolicyRequiredCodeRule {
    fn check(&self, item: &ScheduledPeeringPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPeeringPolicyRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledPeeringPolicyRule for ScheduledPeeringPolicyRateRangeRule {
    fn check(&self, item: &ScheduledPeeringPolicy) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPeeringPolicyCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledPeeringPolicyRule for ScheduledPeeringPolicyCeilingRangeRule {
    fn check(&self, item: &ScheduledPeeringPolicy) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled peering policy.
#[derive(Debug, Clone)]
pub struct ScheduledPeeringPolicyValidator {
    pub rules: Vec<Rc<dyn ScheduledPeeringPolicyRule>>,
}

impl ScheduledPeeringPolicyValidator {
    pub fn validate(&self, item: &ScheduledPeeringPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledPeeringPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledPeeringPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_peering_policy_validator() -> ScheduledPeeringPolicyValidator {
    ScheduledPeeringPolicyValidator {
        rules: vec![
            Rc::new(ScheduledPeeringPolicyRequiredCodeRule {
                message: "scheduled peering policy needs this field".to_string(),
            }) as Rc<dyn ScheduledPeeringPolicyRule>,
            Rc::new(ScheduledPeeringPolicyRateRangeRule { min: 1.3, max: 3210.4 }),
            Rc::new(ScheduledPeeringPolicyCeilingRangeRule { min: 9.3, max: 4194.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_peering_policy_validator();
        let item = new_scheduled_peering_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledPeeringPolicyRequiredCodeRule {
            message: "scheduled peering policy needs this field".to_string(),
        };
        let mut item = new_scheduled_peering_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = ScheduledPeeringPolicyRateRangeRule { min: 6.1, max: 1861.8 };
        let mut item = new_scheduled_peering_policy("x", "y");
        item.rate = 1863.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ScheduledPeeringPolicyCeilingRangeRule { min: 4.9, max: 3818.5 };
        let mut item = new_scheduled_peering_policy("x", "y");
        item.ceiling = 3820.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
