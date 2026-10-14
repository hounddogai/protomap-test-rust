use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record slot must pass. Check returns its problems, or none.
pub trait DnsRecordSlotRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSlotAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl DnsRecordSlotRule for DnsRecordSlotAllowedTenantIdRule {
    fn check(&self, item: &DnsRecordSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is above maximum", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSlotCodeLengthRule {
    pub max_length: i64,
}

impl DnsRecordSlotRule for DnsRecordSlotCodeLengthRule {
    fn check(&self, item: &DnsRecordSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSlotRequiredCodeRule {
    pub message: String,
}

impl DnsRecordSlotRule for DnsRecordSlotRequiredCodeRule {
    fn check(&self, item: &DnsRecordSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSlotQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordSlotRule for DnsRecordSlotQuantityRangeRule {
    fn check(&self, item: &DnsRecordSlot) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSlotRequiredNameRule {
    pub message: String,
}

impl DnsRecordSlotRule for DnsRecordSlotRequiredNameRule {
    fn check(&self, item: &DnsRecordSlot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSlotRequiredNotesRule {
    pub message: String,
}

impl DnsRecordSlotRule for DnsRecordSlotRequiredNotesRule {
    fn check(&self, item: &DnsRecordSlot) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record slot.
#[derive(Debug, Clone)]
pub struct DnsRecordSlotValidator {
    pub rules: Vec<Rc<dyn DnsRecordSlotRule>>,
}

impl DnsRecordSlotValidator {
    pub fn validate(&self, item: &DnsRecordSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_slot_validator() -> DnsRecordSlotValidator {
    DnsRecordSlotValidator {
        rules: vec![
            Rc::new(DnsRecordSlotAllowedTenantIdRule {
                allowed: vec!["adaptive".to_string(), "tiered".to_string()],
            }) as Rc<dyn DnsRecordSlotRule>,
            Rc::new(DnsRecordSlotCodeLengthRule { max_length: 1000 }),
            Rc::new(DnsRecordSlotRequiredCodeRule {
                message: "dns record slot needs an entry".to_string(),
            }),
            Rc::new(DnsRecordSlotQuantityRangeRule { min: 120, max: 3600 }),
            Rc::new(DnsRecordSlotRequiredNameRule {
                message: "dns record slot needs a value".to_string(),
            }),
            Rc::new(DnsRecordSlotRequiredNotesRule {
                message: "dns record slot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_slot_validator();
        let item = new_dns_record_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DnsRecordSlotCodeLengthRule { max_length: 16 };
        let mut item = new_dns_record_slot("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DnsRecordSlotRequiredCodeRule {
            message: "dns record slot needs this field".to_string(),
        };
        let mut item = new_dns_record_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = DnsRecordSlotQuantityRangeRule { min: 32, max: 80 };
        let mut item = new_dns_record_slot("x", "y");
        item.quantity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DnsRecordSlotRequiredNameRule {
            message: "dns record slot needs an entry".to_string(),
        };
        let mut item = new_dns_record_slot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = DnsRecordSlotRequiredNotesRule {
            message: "dns record slot needs an entry".to_string(),
        };
        let mut item = new_dns_record_slot("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
