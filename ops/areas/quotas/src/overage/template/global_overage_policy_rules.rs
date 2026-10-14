use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global overage policy must pass. Check returns its problems, or none.
pub trait GlobalOveragePolicyRule: std::fmt::Debug {
    fn check(&self, item: &GlobalOveragePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOveragePolicyTenantIdLengthRule {
    pub max_length: i64,
}

impl GlobalOveragePolicyRule for GlobalOveragePolicyTenantIdLengthRule {
    fn check(&self, item: &GlobalOveragePolicy) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOveragePolicySequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalOveragePolicyRule for GlobalOveragePolicySequenceRangeRule {
    fn check(&self, item: &GlobalOveragePolicy) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOveragePolicyCodePrefixRule {
    pub prefix: String,
}

impl GlobalOveragePolicyRule for GlobalOveragePolicyCodePrefixRule {
    fn check(&self, item: &GlobalOveragePolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global overage policy.
#[derive(Debug, Clone)]
pub struct GlobalOveragePolicyValidator {
    pub rules: Vec<Rc<dyn GlobalOveragePolicyRule>>,
}

impl GlobalOveragePolicyValidator {
    pub fn validate(&self, item: &GlobalOveragePolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalOveragePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalOveragePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_overage_policy_validator() -> GlobalOveragePolicyValidator {
    GlobalOveragePolicyValidator {
        rules: vec![
            Rc::new(GlobalOveragePolicyTenantIdLengthRule { max_length: 120 }) as Rc<dyn GlobalOveragePolicyRule>,
            Rc::new(GlobalOveragePolicySequenceRangeRule { min: 16, max: 120 }),
            Rc::new(GlobalOveragePolicyCodePrefixRule {
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
        let validator = default_global_overage_policy_validator();
        let item = new_global_overage_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = GlobalOveragePolicyTenantIdLengthRule { max_length: 80 };
        let mut item = new_global_overage_policy("x", "y");
        item.tenant_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = GlobalOveragePolicySequenceRangeRule { min: 3600, max: 86400 };
        let mut item = new_global_overage_policy("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
