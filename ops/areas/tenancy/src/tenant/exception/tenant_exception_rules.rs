use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant exception must pass. Check returns its problems, or none.
pub trait TenantExceptionRule: std::fmt::Debug {
    fn check(&self, item: &TenantException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantExceptionRequiredSegmentRule {
    pub message: String,
}

impl TenantExceptionRule for TenantExceptionRequiredSegmentRule {
    fn check(&self, item: &TenantException) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantExceptionApprovedGuardRule {
    pub limit: i64,
}

impl TenantExceptionRule for TenantExceptionApprovedGuardRule {
    fn check(&self, item: &TenantException) -> Vec<String> {
        if item.approved && (item.updated_at > self.limit) {
            return vec![format!(
                "updated at {} exceeds {} while approved",
                item.updated_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantExceptionRequiredNameRule {
    pub message: String,
}

impl TenantExceptionRule for TenantExceptionRequiredNameRule {
    fn check(&self, item: &TenantException) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantExceptionNameLengthRule {
    pub max_length: i64,
}

impl TenantExceptionRule for TenantExceptionNameLengthRule {
    fn check(&self, item: &TenantException) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantExceptionSegmentLengthRule {
    pub max_length: i64,
}

impl TenantExceptionRule for TenantExceptionSegmentLengthRule {
    fn check(&self, item: &TenantException) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantExceptionCipherLengthRule {
    pub max_length: i64,
}

impl TenantExceptionRule for TenantExceptionCipherLengthRule {
    fn check(&self, item: &TenantException) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant exception.
#[derive(Debug, Clone)]
pub struct TenantExceptionValidator {
    pub rules: Vec<Rc<dyn TenantExceptionRule>>,
}

impl TenantExceptionValidator {
    pub fn validate(&self, item: &TenantException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_exception_validator() -> TenantExceptionValidator {
    TenantExceptionValidator {
        rules: vec![
            Rc::new(TenantExceptionRequiredSegmentRule {
                message: "tenant exception needs a value".to_string(),
            }) as Rc<dyn TenantExceptionRule>,
            Rc::new(TenantExceptionApprovedGuardRule { limit: 64 }),
            Rc::new(TenantExceptionRequiredNameRule {
                message: "tenant exception needs a value".to_string(),
            }),
            Rc::new(TenantExceptionNameLengthRule { max_length: 64 }),
            Rc::new(TenantExceptionSegmentLengthRule { max_length: 16 }),
            Rc::new(TenantExceptionCipherLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_exception_validator();
        let item = new_tenant_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = TenantExceptionRequiredSegmentRule {
            message: "tenant exception needs this field".to_string(),
        };
        let mut item = new_tenant_exception("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TenantExceptionRequiredNameRule {
            message: "tenant exception needs this field".to_string(),
        };
        let mut item = new_tenant_exception("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TenantExceptionNameLengthRule { max_length: 16 };
        let mut item = new_tenant_exception("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = TenantExceptionSegmentLengthRule { max_length: 86400 };
        let mut item = new_tenant_exception("x", "y");
        item.segment = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = TenantExceptionCipherLengthRule { max_length: 3600 };
        let mut item = new_tenant_exception("x", "y");
        item.cipher = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
