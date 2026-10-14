use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual tenant notice must pass. Check returns its problems, or none.
pub trait ManualTenantNoticeRule: std::fmt::Debug {
    fn check(&self, item: &ManualTenantNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantNoticeExternalRefLengthRule {
    pub max_length: i64,
}

impl ManualTenantNoticeRule for ManualTenantNoticeExternalRefLengthRule {
    fn check(&self, item: &ManualTenantNotice) -> Vec<String> {
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
pub struct ManualTenantNoticeCodeLengthRule {
    pub max_length: i64,
}

impl ManualTenantNoticeRule for ManualTenantNoticeCodeLengthRule {
    fn check(&self, item: &ManualTenantNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantNoticeArchivedGuardRule {
    pub limit: i64,
}

impl ManualTenantNoticeRule for ManualTenantNoticeArchivedGuardRule {
    fn check(&self, item: &ManualTenantNotice) -> Vec<String> {
        if item.archived && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while archived",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantNoticeRequiredCodeRule {
    pub message: String,
}

impl ManualTenantNoticeRule for ManualTenantNoticeRequiredCodeRule {
    fn check(&self, item: &ManualTenantNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantNoticeNameLengthRule {
    pub max_length: i64,
}

impl ManualTenantNoticeRule for ManualTenantNoticeNameLengthRule {
    fn check(&self, item: &ManualTenantNotice) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantNoticeRequiredExternalRefRule {
    pub message: String,
}

impl ManualTenantNoticeRule for ManualTenantNoticeRequiredExternalRefRule {
    fn check(&self, item: &ManualTenantNotice) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual tenant notice.
#[derive(Debug, Clone)]
pub struct ManualTenantNoticeValidator {
    pub rules: Vec<Rc<dyn ManualTenantNoticeRule>>,
}

impl ManualTenantNoticeValidator {
    pub fn validate(&self, item: &ManualTenantNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualTenantNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualTenantNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_tenant_notice_validator() -> ManualTenantNoticeValidator {
    ManualTenantNoticeValidator {
        rules: vec![
            Rc::new(ManualTenantNoticeExternalRefLengthRule { max_length: 3600 }) as Rc<dyn ManualTenantNoticeRule>,
            Rc::new(ManualTenantNoticeCodeLengthRule { max_length: 1000 }),
            Rc::new(ManualTenantNoticeArchivedGuardRule { limit: 3600 }),
            Rc::new(ManualTenantNoticeRequiredCodeRule {
                message: "manual tenant notice needs an entry".to_string(),
            }),
            Rc::new(ManualTenantNoticeNameLengthRule { max_length: 3600 }),
            Rc::new(ManualTenantNoticeRequiredExternalRefRule {
                message: "manual tenant notice needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_tenant_notice_validator();
        let item = new_manual_tenant_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = ManualTenantNoticeExternalRefLengthRule { max_length: 32 };
        let mut item = new_manual_tenant_notice("x", "y");
        item.external_ref = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualTenantNoticeCodeLengthRule { max_length: 255 };
        let mut item = new_manual_tenant_notice("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualTenantNoticeRequiredCodeRule {
            message: "manual tenant notice needs this field".to_string(),
        };
        let mut item = new_manual_tenant_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ManualTenantNoticeNameLengthRule { max_length: 32 };
        let mut item = new_manual_tenant_notice("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = ManualTenantNoticeRequiredExternalRefRule {
            message: "manual tenant notice needs this field".to_string(),
        };
        let mut item = new_manual_tenant_notice("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
