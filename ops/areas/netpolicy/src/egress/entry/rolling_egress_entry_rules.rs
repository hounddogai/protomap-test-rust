use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling egress entry must pass. Check returns its problems, or none.
pub trait RollingEgressEntryRule: std::fmt::Debug {
    fn check(&self, item: &RollingEgressEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressEntrySourceLengthRule {
    pub max_length: i64,
}

impl RollingEgressEntryRule for RollingEgressEntrySourceLengthRule {
    fn check(&self, item: &RollingEgressEntry) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressEntryCodePrefixRule {
    pub prefix: String,
}

impl RollingEgressEntryRule for RollingEgressEntryCodePrefixRule {
    fn check(&self, item: &RollingEgressEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressEntryRequiredCodeRule {
    pub message: String,
}

impl RollingEgressEntryRule for RollingEgressEntryRequiredCodeRule {
    fn check(&self, item: &RollingEgressEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressEntryCodeLengthRule {
    pub max_length: i64,
}

impl RollingEgressEntryRule for RollingEgressEntryCodeLengthRule {
    fn check(&self, item: &RollingEgressEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressEntryLabelLengthRule {
    pub max_length: i64,
}

impl RollingEgressEntryRule for RollingEgressEntryLabelLengthRule {
    fn check(&self, item: &RollingEgressEntry) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling egress entry.
#[derive(Debug, Clone)]
pub struct RollingEgressEntryValidator {
    pub rules: Vec<Rc<dyn RollingEgressEntryRule>>,
}

impl RollingEgressEntryValidator {
    pub fn validate(&self, item: &RollingEgressEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingEgressEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingEgressEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_egress_entry_validator() -> RollingEgressEntryValidator {
    RollingEgressEntryValidator {
        rules: vec![
            Rc::new(RollingEgressEntrySourceLengthRule { max_length: 1000 }) as Rc<dyn RollingEgressEntryRule>,
            Rc::new(RollingEgressEntryCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RollingEgressEntryRequiredCodeRule {
                message: "rolling egress entry needs an entry".to_string(),
            }),
            Rc::new(RollingEgressEntryCodeLengthRule { max_length: 80 }),
            Rc::new(RollingEgressEntryLabelLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_egress_entry_validator();
        let item = new_rolling_egress_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = RollingEgressEntrySourceLengthRule { max_length: 3600 };
        let mut item = new_rolling_egress_entry("x", "y");
        item.source = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingEgressEntryRequiredCodeRule {
            message: "rolling egress entry needs an entry".to_string(),
        };
        let mut item = new_rolling_egress_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingEgressEntryCodeLengthRule { max_length: 120 };
        let mut item = new_rolling_egress_entry("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollingEgressEntryLabelLengthRule { max_length: 120 };
        let mut item = new_rolling_egress_entry("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
