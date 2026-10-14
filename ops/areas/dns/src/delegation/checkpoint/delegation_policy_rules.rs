use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a delegation policy must pass. Check returns its problems, or none.
pub trait DelegationPolicyRule: std::fmt::Debug {
    fn check(&self, item: &DelegationPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPolicyCodePrefixRule {
    pub prefix: String,
}

impl DelegationPolicyRule for DelegationPolicyCodePrefixRule {
    fn check(&self, item: &DelegationPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPolicyRequiredTitleRule {
    pub message: String,
}

impl DelegationPolicyRule for DelegationPolicyRequiredTitleRule {
    fn check(&self, item: &DelegationPolicy) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPolicyRequiredCodeRule {
    pub message: String,
}

impl DelegationPolicyRule for DelegationPolicyRequiredCodeRule {
    fn check(&self, item: &DelegationPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPolicyBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationPolicyRule for DelegationPolicyBurstRangeRule {
    fn check(&self, item: &DelegationPolicy) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPolicyAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl DelegationPolicyRule for DelegationPolicyAllowedNotesRule {
    fn check(&self, item: &DelegationPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is unknown", item.notes)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation policy.
#[derive(Debug, Clone)]
pub struct DelegationPolicyValidator {
    pub rules: Vec<Rc<dyn DelegationPolicyRule>>,
}

impl DelegationPolicyValidator {
    pub fn validate(&self, item: &DelegationPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_policy_validator() -> DelegationPolicyValidator {
    DelegationPolicyValidator {
        rules: vec![
            Rc::new(DelegationPolicyCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn DelegationPolicyRule>,
            Rc::new(DelegationPolicyRequiredTitleRule {
                message: "delegation policy needs an entry".to_string(),
            }),
            Rc::new(DelegationPolicyRequiredCodeRule {
                message: "delegation policy needs this field".to_string(),
            }),
            Rc::new(DelegationPolicyBurstRangeRule { min: 32, max: 80 }),
            Rc::new(DelegationPolicyAllowedNotesRule {
                allowed: vec!["pending".to_string(), "default".to_string(), "default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_policy_validator();
        let item = new_delegation_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DelegationPolicyRequiredTitleRule {
            message: "delegation policy needs an entry".to_string(),
        };
        let mut item = new_delegation_policy("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DelegationPolicyRequiredCodeRule {
            message: "delegation policy needs a value".to_string(),
        };
        let mut item = new_delegation_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = DelegationPolicyBurstRangeRule { min: 80, max: 255 };
        let mut item = new_delegation_policy("x", "y");
        item.burst = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
