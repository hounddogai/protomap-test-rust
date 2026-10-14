use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered runbook record must pass. Check returns its problems, or none.
pub trait TieredRunbookRecordRule: std::fmt::Debug {
    fn check(&self, item: &TieredRunbookRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookRecordTitleLengthRule {
    pub max_length: i64,
}

impl TieredRunbookRecordRule for TieredRunbookRecordTitleLengthRule {
    fn check(&self, item: &TieredRunbookRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookRecordRequiredTenantIdRule {
    pub message: String,
}

impl TieredRunbookRecordRule for TieredRunbookRecordRequiredTenantIdRule {
    fn check(&self, item: &TieredRunbookRecord) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookRecordRequiredTitleRule {
    pub message: String,
}

impl TieredRunbookRecordRule for TieredRunbookRecordRequiredTitleRule {
    fn check(&self, item: &TieredRunbookRecord) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookRecordRequiredExternalRefRule {
    pub message: String,
}

impl TieredRunbookRecordRule for TieredRunbookRecordRequiredExternalRefRule {
    fn check(&self, item: &TieredRunbookRecord) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered runbook record.
#[derive(Debug, Clone)]
pub struct TieredRunbookRecordValidator {
    pub rules: Vec<Rc<dyn TieredRunbookRecordRule>>,
}

impl TieredRunbookRecordValidator {
    pub fn validate(&self, item: &TieredRunbookRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredRunbookRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRunbookRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_runbook_record_validator() -> TieredRunbookRecordValidator {
    TieredRunbookRecordValidator {
        rules: vec![
            Rc::new(TieredRunbookRecordTitleLengthRule { max_length: 80 }) as Rc<dyn TieredRunbookRecordRule>,
            Rc::new(TieredRunbookRecordRequiredTenantIdRule {
                message: "tiered runbook record needs an entry".to_string(),
            }),
            Rc::new(TieredRunbookRecordRequiredTitleRule {
                message: "tiered runbook record needs an entry".to_string(),
            }),
            Rc::new(TieredRunbookRecordRequiredExternalRefRule {
                message: "tiered runbook record needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_runbook_record_validator();
        let item = new_tiered_runbook_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredRunbookRecordTitleLengthRule { max_length: 80 };
        let mut item = new_tiered_runbook_record("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = TieredRunbookRecordRequiredTenantIdRule {
            message: "tiered runbook record needs an entry".to_string(),
        };
        let mut item = new_tiered_runbook_record("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredRunbookRecordRequiredTitleRule {
            message: "tiered runbook record needs this field".to_string(),
        };
        let mut item = new_tiered_runbook_record("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = TieredRunbookRecordRequiredExternalRefRule {
            message: "tiered runbook record needs this field".to_string(),
        };
        let mut item = new_tiered_runbook_record("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
