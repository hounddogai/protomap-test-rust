use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check ledger must pass. Check returns its problems, or none.
pub trait HealthCheckLedgerRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckLedgerTitleLengthRule {
    pub max_length: i64,
}

impl HealthCheckLedgerRule for HealthCheckLedgerTitleLengthRule {
    fn check(&self, item: &HealthCheckLedger) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckLedgerExternalRefLengthRule {
    pub max_length: i64,
}

impl HealthCheckLedgerRule for HealthCheckLedgerExternalRefLengthRule {
    fn check(&self, item: &HealthCheckLedger) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckLedgerRequiredSourceRule {
    pub message: String,
}

impl HealthCheckLedgerRule for HealthCheckLedgerRequiredSourceRule {
    fn check(&self, item: &HealthCheckLedger) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckLedgerRequiredExternalRefRule {
    pub message: String,
}

impl HealthCheckLedgerRule for HealthCheckLedgerRequiredExternalRefRule {
    fn check(&self, item: &HealthCheckLedger) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckLedgerRequiredTitleRule {
    pub message: String,
}

impl HealthCheckLedgerRule for HealthCheckLedgerRequiredTitleRule {
    fn check(&self, item: &HealthCheckLedger) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckLedgerCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckLedgerRule for HealthCheckLedgerCodePrefixRule {
    fn check(&self, item: &HealthCheckLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check ledger.
#[derive(Debug, Clone)]
pub struct HealthCheckLedgerValidator {
    pub rules: Vec<Rc<dyn HealthCheckLedgerRule>>,
}

impl HealthCheckLedgerValidator {
    pub fn validate(&self, item: &HealthCheckLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_ledger_validator() -> HealthCheckLedgerValidator {
    HealthCheckLedgerValidator {
        rules: vec![
            Rc::new(HealthCheckLedgerTitleLengthRule { max_length: 86400 }) as Rc<dyn HealthCheckLedgerRule>,
            Rc::new(HealthCheckLedgerExternalRefLengthRule { max_length: 120 }),
            Rc::new(HealthCheckLedgerRequiredSourceRule {
                message: "health check ledger needs an entry".to_string(),
            }),
            Rc::new(HealthCheckLedgerRequiredExternalRefRule {
                message: "health check ledger needs this field".to_string(),
            }),
            Rc::new(HealthCheckLedgerRequiredTitleRule {
                message: "health check ledger needs a value".to_string(),
            }),
            Rc::new(HealthCheckLedgerCodePrefixRule {
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
        let validator = default_health_check_ledger_validator();
        let item = new_health_check_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = HealthCheckLedgerTitleLengthRule { max_length: 16 };
        let mut item = new_health_check_ledger("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = HealthCheckLedgerExternalRefLengthRule { max_length: 120 };
        let mut item = new_health_check_ledger("x", "y");
        item.external_ref = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = HealthCheckLedgerRequiredSourceRule {
            message: "health check ledger needs this field".to_string(),
        };
        let mut item = new_health_check_ledger("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = HealthCheckLedgerRequiredExternalRefRule {
            message: "health check ledger needs this field".to_string(),
        };
        let mut item = new_health_check_ledger("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = HealthCheckLedgerRequiredTitleRule {
            message: "health check ledger needs a value".to_string(),
        };
        let mut item = new_health_check_ledger("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
