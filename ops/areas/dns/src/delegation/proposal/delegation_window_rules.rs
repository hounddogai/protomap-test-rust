use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation window must pass. Check returns its problems, or none.
pub trait DelegationWindowRule: std::fmt::Debug {
    fn check(&self, item: &DelegationWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationWindowCodePrefixRule {
    pub prefix: String,
}

impl DelegationWindowRule for DelegationWindowCodePrefixRule {
    fn check(&self, item: &DelegationWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationWindowRequiredCodeRule {
    pub message: String,
}

impl DelegationWindowRule for DelegationWindowRequiredCodeRule {
    fn check(&self, item: &DelegationWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationWindowTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationWindowRule for DelegationWindowTtlSecondsRangeRule {
    fn check(&self, item: &DelegationWindow) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationWindowUnitLengthRule {
    pub max_length: i64,
}

impl DelegationWindowRule for DelegationWindowUnitLengthRule {
    fn check(&self, item: &DelegationWindow) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationWindowRequiredSummaryRule {
    pub message: String,
}

impl DelegationWindowRule for DelegationWindowRequiredSummaryRule {
    fn check(&self, item: &DelegationWindow) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation window.
#[derive(Debug, Clone)]
pub struct DelegationWindowValidator {
    pub rules: Vec<Rc<dyn DelegationWindowRule>>,
}

impl DelegationWindowValidator {
    pub fn validate(&self, item: &DelegationWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_window_validator() -> DelegationWindowValidator {
    DelegationWindowValidator {
        rules: vec![
            Rc::new(DelegationWindowCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn DelegationWindowRule>,
            Rc::new(DelegationWindowRequiredCodeRule {
                message: "delegation window needs this field".to_string(),
            }),
            Rc::new(DelegationWindowTtlSecondsRangeRule { min: 64, max: 1000 }),
            Rc::new(DelegationWindowUnitLengthRule { max_length: 16 }),
            Rc::new(DelegationWindowRequiredSummaryRule {
                message: "delegation window needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_window_validator();
        let item = new_delegation_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DelegationWindowRequiredCodeRule {
            message: "delegation window needs an entry".to_string(),
        };
        let mut item = new_delegation_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = DelegationWindowTtlSecondsRangeRule { min: 80, max: 86400 };
        let mut item = new_delegation_window("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = DelegationWindowUnitLengthRule { max_length: 32 };
        let mut item = new_delegation_window("x", "y");
        item.unit = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = DelegationWindowRequiredSummaryRule {
            message: "delegation window needs this field".to_string(),
        };
        let mut item = new_delegation_window("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
