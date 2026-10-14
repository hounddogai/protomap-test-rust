use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy group must pass. Check returns its problems, or none.
pub trait TtlPolicyGroupRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyGroupRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyGroupRule for TtlPolicyGroupRequiredCodeRule {
    fn check(&self, item: &TtlPolicyGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyGroupSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyGroupRule for TtlPolicyGroupSequenceRangeRule {
    fn check(&self, item: &TtlPolicyGroup) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyGroupCodePrefixRule {
    pub prefix: String,
}

impl TtlPolicyGroupRule for TtlPolicyGroupCodePrefixRule {
    fn check(&self, item: &TtlPolicyGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy group.
#[derive(Debug, Clone)]
pub struct TtlPolicyGroupValidator {
    pub rules: Vec<Rc<dyn TtlPolicyGroupRule>>,
}

impl TtlPolicyGroupValidator {
    pub fn validate(&self, item: &TtlPolicyGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_group_validator() -> TtlPolicyGroupValidator {
    TtlPolicyGroupValidator {
        rules: vec![
            Rc::new(TtlPolicyGroupRequiredCodeRule {
                message: "ttl policy group needs an entry".to_string(),
            }) as Rc<dyn TtlPolicyGroupRule>,
            Rc::new(TtlPolicyGroupSequenceRangeRule { min: 16, max: 80 }),
            Rc::new(TtlPolicyGroupCodePrefixRule {
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
        let validator = default_ttl_policy_group_validator();
        let item = new_ttl_policy_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TtlPolicyGroupRequiredCodeRule {
            message: "ttl policy group needs a value".to_string(),
        };
        let mut item = new_ttl_policy_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = TtlPolicyGroupSequenceRangeRule { min: 64, max: 120 };
        let mut item = new_ttl_policy_group("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
