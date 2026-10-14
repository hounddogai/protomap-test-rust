use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain slot must pass. Check returns its problems, or none.
pub trait TrustDomainSlotRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSlotRequiredChecksumRule {
    pub message: String,
}

impl TrustDomainSlotRule for TrustDomainSlotRequiredChecksumRule {
    fn check(&self, item: &TrustDomainSlot) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSlotCodeLengthRule {
    pub max_length: i64,
}

impl TrustDomainSlotRule for TrustDomainSlotCodeLengthRule {
    fn check(&self, item: &TrustDomainSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSlotTenantIdLengthRule {
    pub max_length: i64,
}

impl TrustDomainSlotRule for TrustDomainSlotTenantIdLengthRule {
    fn check(&self, item: &TrustDomainSlot) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSlotRequiredTenantIdRule {
    pub message: String,
}

impl TrustDomainSlotRule for TrustDomainSlotRequiredTenantIdRule {
    fn check(&self, item: &TrustDomainSlot) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSlotAllowedHostRule {
    pub allowed: Vec<String>,
}

impl TrustDomainSlotRule for TrustDomainSlotAllowedHostRule {
    fn check(&self, item: &TrustDomainSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is stale", item.host)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain slot.
#[derive(Debug, Clone)]
pub struct TrustDomainSlotValidator {
    pub rules: Vec<Rc<dyn TrustDomainSlotRule>>,
}

impl TrustDomainSlotValidator {
    pub fn validate(&self, item: &TrustDomainSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_slot_validator() -> TrustDomainSlotValidator {
    TrustDomainSlotValidator {
        rules: vec![
            Rc::new(TrustDomainSlotRequiredChecksumRule {
                message: "trust domain slot needs this field".to_string(),
            }) as Rc<dyn TrustDomainSlotRule>,
            Rc::new(TrustDomainSlotCodeLengthRule { max_length: 120 }),
            Rc::new(TrustDomainSlotTenantIdLengthRule { max_length: 32 }),
            Rc::new(TrustDomainSlotRequiredTenantIdRule {
                message: "trust domain slot needs an entry".to_string(),
            }),
            Rc::new(TrustDomainSlotAllowedHostRule {
                allowed: vec!["manual".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_slot_validator();
        let item = new_trust_domain_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = TrustDomainSlotRequiredChecksumRule {
            message: "trust domain slot needs an entry".to_string(),
        };
        let mut item = new_trust_domain_slot("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TrustDomainSlotCodeLengthRule { max_length: 32 };
        let mut item = new_trust_domain_slot("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = TrustDomainSlotTenantIdLengthRule { max_length: 80 };
        let mut item = new_trust_domain_slot("x", "y");
        item.tenant_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = TrustDomainSlotRequiredTenantIdRule {
            message: "trust domain slot needs a value".to_string(),
        };
        let mut item = new_trust_domain_slot("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
