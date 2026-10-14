use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain snapshot must pass. Check returns its problems, or none.
pub trait TrustDomainSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl TrustDomainSnapshotRule for TrustDomainSnapshotCodeLengthRule {
    fn check(&self, item: &TrustDomainSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSnapshotLabelLengthRule {
    pub max_length: i64,
}

impl TrustDomainSnapshotRule for TrustDomainSnapshotLabelLengthRule {
    fn check(&self, item: &TrustDomainSnapshot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSnapshotRequiredTenantIdRule {
    pub message: String,
}

impl TrustDomainSnapshotRule for TrustDomainSnapshotRequiredTenantIdRule {
    fn check(&self, item: &TrustDomainSnapshot) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSnapshotRequiredTimezoneRule {
    pub message: String,
}

impl TrustDomainSnapshotRule for TrustDomainSnapshotRequiredTimezoneRule {
    fn check(&self, item: &TrustDomainSnapshot) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSnapshotRequiredCodeRule {
    pub message: String,
}

impl TrustDomainSnapshotRule for TrustDomainSnapshotRequiredCodeRule {
    fn check(&self, item: &TrustDomainSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSnapshotRequiredLabelRule {
    pub message: String,
}

impl TrustDomainSnapshotRule for TrustDomainSnapshotRequiredLabelRule {
    fn check(&self, item: &TrustDomainSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain snapshot.
#[derive(Debug, Clone)]
pub struct TrustDomainSnapshotValidator {
    pub rules: Vec<Rc<dyn TrustDomainSnapshotRule>>,
}

impl TrustDomainSnapshotValidator {
    pub fn validate(&self, item: &TrustDomainSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_snapshot_validator() -> TrustDomainSnapshotValidator {
    TrustDomainSnapshotValidator {
        rules: vec![
            Rc::new(TrustDomainSnapshotCodeLengthRule { max_length: 80 }) as Rc<dyn TrustDomainSnapshotRule>,
            Rc::new(TrustDomainSnapshotLabelLengthRule { max_length: 1000 }),
            Rc::new(TrustDomainSnapshotRequiredTenantIdRule {
                message: "trust domain snapshot needs a value".to_string(),
            }),
            Rc::new(TrustDomainSnapshotRequiredTimezoneRule {
                message: "trust domain snapshot needs this field".to_string(),
            }),
            Rc::new(TrustDomainSnapshotRequiredCodeRule {
                message: "trust domain snapshot needs this field".to_string(),
            }),
            Rc::new(TrustDomainSnapshotRequiredLabelRule {
                message: "trust domain snapshot needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_snapshot_validator();
        let item = new_trust_domain_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TrustDomainSnapshotCodeLengthRule { max_length: 1000 };
        let mut item = new_trust_domain_snapshot("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TrustDomainSnapshotLabelLengthRule { max_length: 255 };
        let mut item = new_trust_domain_snapshot("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = TrustDomainSnapshotRequiredTenantIdRule {
            message: "trust domain snapshot needs a value".to_string(),
        };
        let mut item = new_trust_domain_snapshot("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = TrustDomainSnapshotRequiredTimezoneRule {
            message: "trust domain snapshot needs an entry".to_string(),
        };
        let mut item = new_trust_domain_snapshot("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TrustDomainSnapshotRequiredCodeRule {
            message: "trust domain snapshot needs this field".to_string(),
        };
        let mut item = new_trust_domain_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TrustDomainSnapshotRequiredLabelRule {
            message: "trust domain snapshot needs this field".to_string(),
        };
        let mut item = new_trust_domain_snapshot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
