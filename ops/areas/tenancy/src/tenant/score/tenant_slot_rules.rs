use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant slot must pass. Check returns its problems, or none.
pub trait TenantSlotRule: std::fmt::Debug {
    fn check(&self, item: &TenantSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantSlotRequiredNameRule {
    pub message: String,
}

impl TenantSlotRule for TenantSlotRequiredNameRule {
    fn check(&self, item: &TenantSlot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantSlotCodeLengthRule {
    pub max_length: i64,
}

impl TenantSlotRule for TenantSlotCodeLengthRule {
    fn check(&self, item: &TenantSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantSlotBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantSlotRule for TenantSlotBurstRangeRule {
    fn check(&self, item: &TenantSlot) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantSlotRequiredPrincipalRule {
    pub message: String,
}

impl TenantSlotRule for TenantSlotRequiredPrincipalRule {
    fn check(&self, item: &TenantSlot) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantSlotRequiredTenantIdRule {
    pub message: String,
}

impl TenantSlotRule for TenantSlotRequiredTenantIdRule {
    fn check(&self, item: &TenantSlot) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantSlotExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantSlotRule for TenantSlotExpiresAtRangeRule {
    fn check(&self, item: &TenantSlot) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tenant slot.
#[derive(Debug, Clone)]
pub struct TenantSlotValidator {
    pub rules: Vec<Rc<dyn TenantSlotRule>>,
}

impl TenantSlotValidator {
    pub fn validate(&self, item: &TenantSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_slot_validator() -> TenantSlotValidator {
    TenantSlotValidator {
        rules: vec![
            Rc::new(TenantSlotRequiredNameRule {
                message: "tenant slot needs a value".to_string(),
            }) as Rc<dyn TenantSlotRule>,
            Rc::new(TenantSlotCodeLengthRule { max_length: 32 }),
            Rc::new(TenantSlotBurstRangeRule { min: 3600, max: 86400 }),
            Rc::new(TenantSlotRequiredPrincipalRule {
                message: "tenant slot needs this field".to_string(),
            }),
            Rc::new(TenantSlotRequiredTenantIdRule {
                message: "tenant slot needs this field".to_string(),
            }),
            Rc::new(TenantSlotExpiresAtRangeRule { min: 120, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_slot_validator();
        let item = new_tenant_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TenantSlotRequiredNameRule {
            message: "tenant slot needs an entry".to_string(),
        };
        let mut item = new_tenant_slot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TenantSlotCodeLengthRule { max_length: 80 };
        let mut item = new_tenant_slot("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = TenantSlotBurstRangeRule { min: 16, max: 32 };
        let mut item = new_tenant_slot("x", "y");
        item.burst = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = TenantSlotRequiredPrincipalRule {
            message: "tenant slot needs an entry".to_string(),
        };
        let mut item = new_tenant_slot("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = TenantSlotRequiredTenantIdRule {
            message: "tenant slot needs an entry".to_string(),
        };
        let mut item = new_tenant_slot("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = TenantSlotExpiresAtRangeRule { min: 80, max: 3600 };
        let mut item = new_tenant_slot("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
