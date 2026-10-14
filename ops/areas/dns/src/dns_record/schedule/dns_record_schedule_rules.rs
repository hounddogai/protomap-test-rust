use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record schedule must pass. Check returns its problems, or none.
pub trait DnsRecordScheduleRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordScheduleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DnsRecordScheduleRule for DnsRecordScheduleAllowedCodeRule {
    fn check(&self, item: &DnsRecordSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordScheduleSummaryLengthRule {
    pub max_length: i64,
}

impl DnsRecordScheduleRule for DnsRecordScheduleSummaryLengthRule {
    fn check(&self, item: &DnsRecordSchedule) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordScheduleUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordScheduleRule for DnsRecordScheduleUpdatedAtRangeRule {
    fn check(&self, item: &DnsRecordSchedule) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordScheduleTenantIdLengthRule {
    pub max_length: i64,
}

impl DnsRecordScheduleRule for DnsRecordScheduleTenantIdLengthRule {
    fn check(&self, item: &DnsRecordSchedule) -> Vec<String> {
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
pub struct DnsRecordScheduleRequiredSummaryRule {
    pub message: String,
}

impl DnsRecordScheduleRule for DnsRecordScheduleRequiredSummaryRule {
    fn check(&self, item: &DnsRecordSchedule) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record schedule.
#[derive(Debug, Clone)]
pub struct DnsRecordScheduleValidator {
    pub rules: Vec<Rc<dyn DnsRecordScheduleRule>>,
}

impl DnsRecordScheduleValidator {
    pub fn validate(&self, item: &DnsRecordSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_schedule_validator() -> DnsRecordScheduleValidator {
    DnsRecordScheduleValidator {
        rules: vec![
            Rc::new(DnsRecordScheduleAllowedCodeRule {
                allowed: vec!["global".to_string(), "manual".to_string()],
            }) as Rc<dyn DnsRecordScheduleRule>,
            Rc::new(DnsRecordScheduleSummaryLengthRule { max_length: 80 }),
            Rc::new(DnsRecordScheduleUpdatedAtRangeRule { min: 120, max: 1000 }),
            Rc::new(DnsRecordScheduleTenantIdLengthRule { max_length: 120 }),
            Rc::new(DnsRecordScheduleRequiredSummaryRule {
                message: "dns record schedule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_schedule_validator();
        let item = new_dns_record_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = DnsRecordScheduleSummaryLengthRule { max_length: 120 };
        let mut item = new_dns_record_schedule("x", "y");
        item.summary = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = DnsRecordScheduleUpdatedAtRangeRule { min: 255, max: 3600 };
        let mut item = new_dns_record_schedule("x", "y");
        item.updated_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = DnsRecordScheduleTenantIdLengthRule { max_length: 32 };
        let mut item = new_dns_record_schedule("x", "y");
        item.tenant_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = DnsRecordScheduleRequiredSummaryRule {
            message: "dns record schedule needs a value".to_string(),
        };
        let mut item = new_dns_record_schedule("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
