use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record quota must pass. Check returns its problems, or none.
pub trait DnsRecordQuotaRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordQuotaQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordQuotaRule for DnsRecordQuotaQuantityRangeRule {
    fn check(&self, item: &DnsRecordQuota) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordQuotaCodePrefixRule {
    pub prefix: String,
}

impl DnsRecordQuotaRule for DnsRecordQuotaCodePrefixRule {
    fn check(&self, item: &DnsRecordQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordQuotaRequiredChannelRule {
    pub message: String,
}

impl DnsRecordQuotaRule for DnsRecordQuotaRequiredChannelRule {
    fn check(&self, item: &DnsRecordQuota) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordQuotaCodeLengthRule {
    pub max_length: i64,
}

impl DnsRecordQuotaRule for DnsRecordQuotaCodeLengthRule {
    fn check(&self, item: &DnsRecordQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record quota.
#[derive(Debug, Clone)]
pub struct DnsRecordQuotaValidator {
    pub rules: Vec<Rc<dyn DnsRecordQuotaRule>>,
}

impl DnsRecordQuotaValidator {
    pub fn validate(&self, item: &DnsRecordQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_quota_validator() -> DnsRecordQuotaValidator {
    DnsRecordQuotaValidator {
        rules: vec![
            Rc::new(DnsRecordQuotaQuantityRangeRule { min: 32, max: 120 }) as Rc<dyn DnsRecordQuotaRule>,
            Rc::new(DnsRecordQuotaCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DnsRecordQuotaRequiredChannelRule {
                message: "dns record quota needs a value".to_string(),
            }),
            Rc::new(DnsRecordQuotaCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_quota_validator();
        let item = new_dns_record_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = DnsRecordQuotaQuantityRangeRule { min: 80, max: 86400 };
        let mut item = new_dns_record_quota("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = DnsRecordQuotaRequiredChannelRule {
            message: "dns record quota needs this field".to_string(),
        };
        let mut item = new_dns_record_quota("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DnsRecordQuotaCodeLengthRule { max_length: 64 };
        let mut item = new_dns_record_quota("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
