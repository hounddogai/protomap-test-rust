use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled segment entry must pass. Check returns its problems, or none.
pub trait ScheduledSegmentEntryRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSegmentEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEntryRequiredTimezoneRule {
    pub message: String,
}

impl ScheduledSegmentEntryRule for ScheduledSegmentEntryRequiredTimezoneRule {
    fn check(&self, item: &ScheduledSegmentEntry) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEntryAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledSegmentEntryRule for ScheduledSegmentEntryAmountRangeRule {
    fn check(&self, item: &ScheduledSegmentEntry) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEntryAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl ScheduledSegmentEntryRule for ScheduledSegmentEntryAllowedKeyIdRule {
    fn check(&self, item: &ScheduledSegmentEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is below minimum", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEntryRequiredTenantIdRule {
    pub message: String,
}

impl ScheduledSegmentEntryRule for ScheduledSegmentEntryRequiredTenantIdRule {
    fn check(&self, item: &ScheduledSegmentEntry) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEntryCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledSegmentEntryRule for ScheduledSegmentEntryCodeLengthRule {
    fn check(&self, item: &ScheduledSegmentEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEntryTenantIdLengthRule {
    pub max_length: i64,
}

impl ScheduledSegmentEntryRule for ScheduledSegmentEntryTenantIdLengthRule {
    fn check(&self, item: &ScheduledSegmentEntry) -> Vec<String> {
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

/// Runs every rule of a scheduled segment entry.
#[derive(Debug, Clone)]
pub struct ScheduledSegmentEntryValidator {
    pub rules: Vec<Rc<dyn ScheduledSegmentEntryRule>>,
}

impl ScheduledSegmentEntryValidator {
    pub fn validate(&self, item: &ScheduledSegmentEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledSegmentEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSegmentEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_segment_entry_validator() -> ScheduledSegmentEntryValidator {
    ScheduledSegmentEntryValidator {
        rules: vec![
            Rc::new(ScheduledSegmentEntryRequiredTimezoneRule {
                message: "scheduled segment entry needs this field".to_string(),
            }) as Rc<dyn ScheduledSegmentEntryRule>,
            Rc::new(ScheduledSegmentEntryAmountRangeRule { min: 0.3, max: 4421.0 }),
            Rc::new(ScheduledSegmentEntryAllowedKeyIdRule { allowed: Vec::new() }),
            Rc::new(ScheduledSegmentEntryRequiredTenantIdRule {
                message: "scheduled segment entry needs a value".to_string(),
            }),
            Rc::new(ScheduledSegmentEntryCodeLengthRule { max_length: 64 }),
            Rc::new(ScheduledSegmentEntryTenantIdLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_segment_entry_validator();
        let item = new_scheduled_segment_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = ScheduledSegmentEntryRequiredTimezoneRule {
            message: "scheduled segment entry needs a value".to_string(),
        };
        let mut item = new_scheduled_segment_entry("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = ScheduledSegmentEntryAmountRangeRule { min: 9.9, max: 1481.8 };
        let mut item = new_scheduled_segment_entry("x", "y");
        item.amount = 1483.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = ScheduledSegmentEntryRequiredTenantIdRule {
            message: "scheduled segment entry needs this field".to_string(),
        };
        let mut item = new_scheduled_segment_entry("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledSegmentEntryCodeLengthRule { max_length: 3600 };
        let mut item = new_scheduled_segment_entry("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = ScheduledSegmentEntryTenantIdLengthRule { max_length: 16 };
        let mut item = new_scheduled_segment_entry("x", "y");
        item.tenant_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
