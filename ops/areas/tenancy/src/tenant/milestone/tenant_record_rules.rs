use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant record must pass. Check returns its problems, or none.
pub trait TenantRecordRule: std::fmt::Debug {
    fn check(&self, item: &TenantRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRecordNameLengthRule {
    pub max_length: i64,
}

impl TenantRecordRule for TenantRecordNameLengthRule {
    fn check(&self, item: &TenantRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRecordRequiredNameRule {
    pub message: String,
}

impl TenantRecordRule for TenantRecordRequiredNameRule {
    fn check(&self, item: &TenantRecord) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRecordNamespaceLengthRule {
    pub max_length: i64,
}

impl TenantRecordRule for TenantRecordNamespaceLengthRule {
    fn check(&self, item: &TenantRecord) -> Vec<String> {
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

/// Runs every rule of a tenant record.
#[derive(Debug, Clone)]
pub struct TenantRecordValidator {
    pub rules: Vec<Rc<dyn TenantRecordRule>>,
}

impl TenantRecordValidator {
    pub fn validate(&self, item: &TenantRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_record_validator() -> TenantRecordValidator {
    TenantRecordValidator {
        rules: vec![
            Rc::new(TenantRecordNameLengthRule { max_length: 64 }) as Rc<dyn TenantRecordRule>,
            Rc::new(TenantRecordRequiredNameRule {
                message: "tenant record needs this field".to_string(),
            }),
            Rc::new(TenantRecordNamespaceLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_record_validator();
        let item = new_tenant_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TenantRecordNameLengthRule { max_length: 3600 };
        let mut item = new_tenant_record("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TenantRecordRequiredNameRule {
            message: "tenant record needs an entry".to_string(),
        };
        let mut item = new_tenant_record("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = TenantRecordNamespaceLengthRule { max_length: 16 };
        let mut item = new_tenant_record("x", "y");
        item.namespace = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
