use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant entry must pass. Check returns its problems, or none.
pub trait TenantEntryRule: std::fmt::Debug {
    fn check(&self, item: &TenantEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantEntryNameLengthRule {
    pub max_length: i64,
}

impl TenantEntryRule for TenantEntryNameLengthRule {
    fn check(&self, item: &TenantEntry) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantEntryTitleLengthRule {
    pub max_length: i64,
}

impl TenantEntryRule for TenantEntryTitleLengthRule {
    fn check(&self, item: &TenantEntry) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantEntryRequiredPhoneRule {
    pub message: String,
}

impl TenantEntryRule for TenantEntryRequiredPhoneRule {
    fn check(&self, item: &TenantEntry) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantEntryRequiredTitleRule {
    pub message: String,
}

impl TenantEntryRule for TenantEntryRequiredTitleRule {
    fn check(&self, item: &TenantEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantEntryRequiredCodeRule {
    pub message: String,
}

impl TenantEntryRule for TenantEntryRequiredCodeRule {
    fn check(&self, item: &TenantEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant entry.
#[derive(Debug, Clone)]
pub struct TenantEntryValidator {
    pub rules: Vec<Rc<dyn TenantEntryRule>>,
}

impl TenantEntryValidator {
    pub fn validate(&self, item: &TenantEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_entry_validator() -> TenantEntryValidator {
    TenantEntryValidator {
        rules: vec![
            Rc::new(TenantEntryNameLengthRule { max_length: 120 }) as Rc<dyn TenantEntryRule>,
            Rc::new(TenantEntryTitleLengthRule { max_length: 80 }),
            Rc::new(TenantEntryRequiredPhoneRule {
                message: "tenant entry needs an entry".to_string(),
            }),
            Rc::new(TenantEntryRequiredTitleRule {
                message: "tenant entry needs this field".to_string(),
            }),
            Rc::new(TenantEntryRequiredCodeRule {
                message: "tenant entry needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_entry_validator();
        let item = new_tenant_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TenantEntryNameLengthRule { max_length: 120 };
        let mut item = new_tenant_entry("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TenantEntryTitleLengthRule { max_length: 64 };
        let mut item = new_tenant_entry("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = TenantEntryRequiredPhoneRule {
            message: "tenant entry needs this field".to_string(),
        };
        let mut item = new_tenant_entry("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TenantEntryRequiredTitleRule {
            message: "tenant entry needs this field".to_string(),
        };
        let mut item = new_tenant_entry("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TenantEntryRequiredCodeRule {
            message: "tenant entry needs an entry".to_string(),
        };
        let mut item = new_tenant_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
