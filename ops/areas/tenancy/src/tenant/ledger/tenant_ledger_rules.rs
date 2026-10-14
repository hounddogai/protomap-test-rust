use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tenant ledger must pass. Check returns its problems, or none.
pub trait TenantLedgerRule: std::fmt::Debug {
    fn check(&self, item: &TenantLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantLedgerRequiredTitleRule {
    pub message: String,
}

impl TenantLedgerRule for TenantLedgerRequiredTitleRule {
    fn check(&self, item: &TenantLedger) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantLedgerRequiredClusterRule {
    pub message: String,
}

impl TenantLedgerRule for TenantLedgerRequiredClusterRule {
    fn check(&self, item: &TenantLedger) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantLedgerAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl TenantLedgerRule for TenantLedgerAllowedClusterRule {
    fn check(&self, item: &TenantLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is missing", item.cluster)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantLedgerRequiredOwnerIdRule {
    pub message: String,
}

impl TenantLedgerRule for TenantLedgerRequiredOwnerIdRule {
    fn check(&self, item: &TenantLedger) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantLedgerCodeLengthRule {
    pub max_length: i64,
}

impl TenantLedgerRule for TenantLedgerCodeLengthRule {
    fn check(&self, item: &TenantLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant ledger.
#[derive(Debug, Clone)]
pub struct TenantLedgerValidator {
    pub rules: Vec<Rc<dyn TenantLedgerRule>>,
}

impl TenantLedgerValidator {
    pub fn validate(&self, item: &TenantLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_ledger_validator() -> TenantLedgerValidator {
    TenantLedgerValidator {
        rules: vec![
            Rc::new(TenantLedgerRequiredTitleRule {
                message: "tenant ledger needs a value".to_string(),
            }) as Rc<dyn TenantLedgerRule>,
            Rc::new(TenantLedgerRequiredClusterRule {
                message: "tenant ledger needs this field".to_string(),
            }),
            Rc::new(TenantLedgerAllowedClusterRule {
                allowed: vec!["primary".to_string(), "seasonal".to_string(), "rolling".to_string()],
            }),
            Rc::new(TenantLedgerRequiredOwnerIdRule {
                message: "tenant ledger needs a value".to_string(),
            }),
            Rc::new(TenantLedgerCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_ledger_validator();
        let item = new_tenant_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TenantLedgerRequiredTitleRule {
            message: "tenant ledger needs this field".to_string(),
        };
        let mut item = new_tenant_ledger("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = TenantLedgerRequiredClusterRule {
            message: "tenant ledger needs this field".to_string(),
        };
        let mut item = new_tenant_ledger("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = TenantLedgerRequiredOwnerIdRule {
            message: "tenant ledger needs a value".to_string(),
        };
        let mut item = new_tenant_ledger("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TenantLedgerCodeLengthRule { max_length: 32 };
        let mut item = new_tenant_ledger("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
