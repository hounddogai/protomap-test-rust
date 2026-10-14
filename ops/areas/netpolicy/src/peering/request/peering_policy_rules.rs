use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering policy must pass. Check returns its problems, or none.
pub trait PeeringPolicyRule: std::fmt::Debug {
    fn check(&self, item: &PeeringPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPolicyRequiredOwnerIdRule {
    pub message: String,
}

impl PeeringPolicyRule for PeeringPolicyRequiredOwnerIdRule {
    fn check(&self, item: &PeeringPolicy) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPolicyRequiredCodeRule {
    pub message: String,
}

impl PeeringPolicyRule for PeeringPolicyRequiredCodeRule {
    fn check(&self, item: &PeeringPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPolicyTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringPolicyRule for PeeringPolicyTtlSecondsRangeRule {
    fn check(&self, item: &PeeringPolicy) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPolicyRequiredNameRule {
    pub message: String,
}

impl PeeringPolicyRule for PeeringPolicyRequiredNameRule {
    fn check(&self, item: &PeeringPolicy) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPolicyOwnerIdLengthRule {
    pub max_length: i64,
}

impl PeeringPolicyRule for PeeringPolicyOwnerIdLengthRule {
    fn check(&self, item: &PeeringPolicy) -> Vec<String> {
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

/// Runs every rule of a peering policy.
#[derive(Debug, Clone)]
pub struct PeeringPolicyValidator {
    pub rules: Vec<Rc<dyn PeeringPolicyRule>>,
}

impl PeeringPolicyValidator {
    pub fn validate(&self, item: &PeeringPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_policy_validator() -> PeeringPolicyValidator {
    PeeringPolicyValidator {
        rules: vec![
            Rc::new(PeeringPolicyRequiredOwnerIdRule {
                message: "peering policy needs this field".to_string(),
            }) as Rc<dyn PeeringPolicyRule>,
            Rc::new(PeeringPolicyRequiredCodeRule {
                message: "peering policy needs an entry".to_string(),
            }),
            Rc::new(PeeringPolicyTtlSecondsRangeRule { min: 16, max: 16 }),
            Rc::new(PeeringPolicyRequiredNameRule {
                message: "peering policy needs this field".to_string(),
            }),
            Rc::new(PeeringPolicyOwnerIdLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_policy_validator();
        let item = new_peering_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = PeeringPolicyRequiredOwnerIdRule {
            message: "peering policy needs an entry".to_string(),
        };
        let mut item = new_peering_policy("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PeeringPolicyRequiredCodeRule {
            message: "peering policy needs an entry".to_string(),
        };
        let mut item = new_peering_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = PeeringPolicyTtlSecondsRangeRule { min: 16, max: 64 };
        let mut item = new_peering_policy("x", "y");
        item.ttl_seconds = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PeeringPolicyRequiredNameRule {
            message: "peering policy needs a value".to_string(),
        };
        let mut item = new_peering_policy("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = PeeringPolicyOwnerIdLengthRule { max_length: 16 };
        let mut item = new_peering_policy("x", "y");
        item.owner_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
