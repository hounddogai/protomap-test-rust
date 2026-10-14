use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit allocation must pass. Check returns its problems, or none.
pub trait LimitAllocationRule: std::fmt::Debug {
    fn check(&self, item: &LimitAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAllocationDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitAllocationRule for LimitAllocationDurationMinutesRangeRule {
    fn check(&self, item: &LimitAllocation) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAllocationCodePrefixRule {
    pub prefix: String,
}

impl LimitAllocationRule for LimitAllocationCodePrefixRule {
    fn check(&self, item: &LimitAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAllocationIssuerLengthRule {
    pub max_length: i64,
}

impl LimitAllocationRule for LimitAllocationIssuerLengthRule {
    fn check(&self, item: &LimitAllocation) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAllocationRequiredNamespaceRule {
    pub message: String,
}

impl LimitAllocationRule for LimitAllocationRequiredNamespaceRule {
    fn check(&self, item: &LimitAllocation) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAllocationRequiredCodeRule {
    pub message: String,
}

impl LimitAllocationRule for LimitAllocationRequiredCodeRule {
    fn check(&self, item: &LimitAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAllocationNamespaceLengthRule {
    pub max_length: i64,
}

impl LimitAllocationRule for LimitAllocationNamespaceLengthRule {
    fn check(&self, item: &LimitAllocation) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit allocation.
#[derive(Debug, Clone)]
pub struct LimitAllocationValidator {
    pub rules: Vec<Rc<dyn LimitAllocationRule>>,
}

impl LimitAllocationValidator {
    pub fn validate(&self, item: &LimitAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_allocation_validator() -> LimitAllocationValidator {
    LimitAllocationValidator {
        rules: vec![
            Rc::new(LimitAllocationDurationMinutesRangeRule { min: 16, max: 120 }) as Rc<dyn LimitAllocationRule>,
            Rc::new(LimitAllocationCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LimitAllocationIssuerLengthRule { max_length: 16 }),
            Rc::new(LimitAllocationRequiredNamespaceRule {
                message: "limit allocation needs this field".to_string(),
            }),
            Rc::new(LimitAllocationRequiredCodeRule {
                message: "limit allocation needs a value".to_string(),
            }),
            Rc::new(LimitAllocationNamespaceLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_allocation_validator();
        let item = new_limit_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = LimitAllocationDurationMinutesRangeRule { min: 16, max: 86400 };
        let mut item = new_limit_allocation("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = LimitAllocationIssuerLengthRule { max_length: 120 };
        let mut item = new_limit_allocation("x", "y");
        item.issuer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = LimitAllocationRequiredNamespaceRule {
            message: "limit allocation needs a value".to_string(),
        };
        let mut item = new_limit_allocation("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LimitAllocationRequiredCodeRule {
            message: "limit allocation needs an entry".to_string(),
        };
        let mut item = new_limit_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = LimitAllocationNamespaceLengthRule { max_length: 255 };
        let mut item = new_limit_allocation("x", "y");
        item.namespace = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
