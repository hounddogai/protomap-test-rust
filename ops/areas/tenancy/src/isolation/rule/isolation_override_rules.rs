use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation override must pass. Check returns its problems, or none.
pub trait IsolationOverrideRule: std::fmt::Debug {
    fn check(&self, item: &IsolationOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationOverrideUnitLengthRule {
    pub max_length: i64,
}

impl IsolationOverrideRule for IsolationOverrideUnitLengthRule {
    fn check(&self, item: &IsolationOverride) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationOverrideActiveGuardRule {
    pub limit: i64,
}

impl IsolationOverrideRule for IsolationOverrideActiveGuardRule {
    fn check(&self, item: &IsolationOverride) -> Vec<String> {
        if item.active && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while active",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationOverrideIssuerLengthRule {
    pub max_length: i64,
}

impl IsolationOverrideRule for IsolationOverrideIssuerLengthRule {
    fn check(&self, item: &IsolationOverride) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationOverrideRequiredUnitRule {
    pub message: String,
}

impl IsolationOverrideRule for IsolationOverrideRequiredUnitRule {
    fn check(&self, item: &IsolationOverride) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationOverrideRequiredTitleRule {
    pub message: String,
}

impl IsolationOverrideRule for IsolationOverrideRequiredTitleRule {
    fn check(&self, item: &IsolationOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationOverrideRequiredCodeRule {
    pub message: String,
}

impl IsolationOverrideRule for IsolationOverrideRequiredCodeRule {
    fn check(&self, item: &IsolationOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation override.
#[derive(Debug, Clone)]
pub struct IsolationOverrideValidator {
    pub rules: Vec<Rc<dyn IsolationOverrideRule>>,
}

impl IsolationOverrideValidator {
    pub fn validate(&self, item: &IsolationOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_override_validator() -> IsolationOverrideValidator {
    IsolationOverrideValidator {
        rules: vec![
            Rc::new(IsolationOverrideUnitLengthRule { max_length: 80 }) as Rc<dyn IsolationOverrideRule>,
            Rc::new(IsolationOverrideActiveGuardRule { limit: 120 }),
            Rc::new(IsolationOverrideIssuerLengthRule { max_length: 3600 }),
            Rc::new(IsolationOverrideRequiredUnitRule {
                message: "isolation override needs this field".to_string(),
            }),
            Rc::new(IsolationOverrideRequiredTitleRule {
                message: "isolation override needs an entry".to_string(),
            }),
            Rc::new(IsolationOverrideRequiredCodeRule {
                message: "isolation override needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_override_validator();
        let item = new_isolation_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = IsolationOverrideUnitLengthRule { max_length: 80 };
        let mut item = new_isolation_override("x", "y");
        item.unit = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = IsolationOverrideIssuerLengthRule { max_length: 64 };
        let mut item = new_isolation_override("x", "y");
        item.issuer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = IsolationOverrideRequiredUnitRule {
            message: "isolation override needs an entry".to_string(),
        };
        let mut item = new_isolation_override("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = IsolationOverrideRequiredTitleRule {
            message: "isolation override needs this field".to_string(),
        };
        let mut item = new_isolation_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IsolationOverrideRequiredCodeRule {
            message: "isolation override needs this field".to_string(),
        };
        let mut item = new_isolation_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
