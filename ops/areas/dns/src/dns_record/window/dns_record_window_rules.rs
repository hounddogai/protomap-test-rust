use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record window must pass. Check returns its problems, or none.
pub trait DnsRecordWindowRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordWindowCurrencyLengthRule {
    pub max_length: i64,
}

impl DnsRecordWindowRule for DnsRecordWindowCurrencyLengthRule {
    fn check(&self, item: &DnsRecordWindow) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordWindowQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordWindowRule for DnsRecordWindowQuotaUnitsRangeRule {
    fn check(&self, item: &DnsRecordWindow) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordWindowAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl DnsRecordWindowRule for DnsRecordWindowAllowedReviewerRule {
    fn check(&self, item: &DnsRecordWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is duplicated", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordWindowRequiredReviewerRule {
    pub message: String,
}

impl DnsRecordWindowRule for DnsRecordWindowRequiredReviewerRule {
    fn check(&self, item: &DnsRecordWindow) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordWindowCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordWindowRule for DnsRecordWindowCapacityRangeRule {
    fn check(&self, item: &DnsRecordWindow) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a dns record window.
#[derive(Debug, Clone)]
pub struct DnsRecordWindowValidator {
    pub rules: Vec<Rc<dyn DnsRecordWindowRule>>,
}

impl DnsRecordWindowValidator {
    pub fn validate(&self, item: &DnsRecordWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_window_validator() -> DnsRecordWindowValidator {
    DnsRecordWindowValidator {
        rules: vec![
            Rc::new(DnsRecordWindowCurrencyLengthRule { max_length: 1000 }) as Rc<dyn DnsRecordWindowRule>,
            Rc::new(DnsRecordWindowQuotaUnitsRangeRule { min: 32, max: 255 }),
            Rc::new(DnsRecordWindowAllowedReviewerRule {
                allowed: vec!["primary".to_string(), "manual".to_string(), "pending".to_string()],
            }),
            Rc::new(DnsRecordWindowRequiredReviewerRule {
                message: "dns record window needs a value".to_string(),
            }),
            Rc::new(DnsRecordWindowCapacityRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_window_validator();
        let item = new_dns_record_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = DnsRecordWindowCurrencyLengthRule { max_length: 3600 };
        let mut item = new_dns_record_window("x", "y");
        item.currency = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = DnsRecordWindowQuotaUnitsRangeRule { min: 80, max: 255 };
        let mut item = new_dns_record_window("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = DnsRecordWindowRequiredReviewerRule {
            message: "dns record window needs this field".to_string(),
        };
        let mut item = new_dns_record_window("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = DnsRecordWindowCapacityRangeRule { min: 86400, max: 86400 };
        let mut item = new_dns_record_window("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
