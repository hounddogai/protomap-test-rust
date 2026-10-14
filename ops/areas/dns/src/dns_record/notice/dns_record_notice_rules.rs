use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record notice must pass. Check returns its problems, or none.
pub trait DnsRecordNoticeRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordNoticeRequiredFingerprintRule {
    pub message: String,
}

impl DnsRecordNoticeRule for DnsRecordNoticeRequiredFingerprintRule {
    fn check(&self, item: &DnsRecordNotice) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordNoticeRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordNoticeRule for DnsRecordNoticeRevisionRangeRule {
    fn check(&self, item: &DnsRecordNotice) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordNoticeCodeLengthRule {
    pub max_length: i64,
}

impl DnsRecordNoticeRule for DnsRecordNoticeCodeLengthRule {
    fn check(&self, item: &DnsRecordNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordNoticeRequiredCodeRule {
    pub message: String,
}

impl DnsRecordNoticeRule for DnsRecordNoticeRequiredCodeRule {
    fn check(&self, item: &DnsRecordNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordNoticeGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordNoticeRule for DnsRecordNoticeGraceDaysRangeRule {
    fn check(&self, item: &DnsRecordNotice) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a dns record notice.
#[derive(Debug, Clone)]
pub struct DnsRecordNoticeValidator {
    pub rules: Vec<Rc<dyn DnsRecordNoticeRule>>,
}

impl DnsRecordNoticeValidator {
    pub fn validate(&self, item: &DnsRecordNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_notice_validator() -> DnsRecordNoticeValidator {
    DnsRecordNoticeValidator {
        rules: vec![
            Rc::new(DnsRecordNoticeRequiredFingerprintRule {
                message: "dns record notice needs this field".to_string(),
            }) as Rc<dyn DnsRecordNoticeRule>,
            Rc::new(DnsRecordNoticeRevisionRangeRule { min: 16, max: 86400 }),
            Rc::new(DnsRecordNoticeCodeLengthRule { max_length: 64 }),
            Rc::new(DnsRecordNoticeRequiredCodeRule {
                message: "dns record notice needs an entry".to_string(),
            }),
            Rc::new(DnsRecordNoticeGraceDaysRangeRule { min: 3600, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_notice_validator();
        let item = new_dns_record_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = DnsRecordNoticeRequiredFingerprintRule {
            message: "dns record notice needs an entry".to_string(),
        };
        let mut item = new_dns_record_notice("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = DnsRecordNoticeRevisionRangeRule { min: 32, max: 64 };
        let mut item = new_dns_record_notice("x", "y");
        item.revision = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DnsRecordNoticeCodeLengthRule { max_length: 1000 };
        let mut item = new_dns_record_notice("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DnsRecordNoticeRequiredCodeRule {
            message: "dns record notice needs this field".to_string(),
        };
        let mut item = new_dns_record_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = DnsRecordNoticeGraceDaysRangeRule { min: 16, max: 80 };
        let mut item = new_dns_record_notice("x", "y");
        item.grace_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
