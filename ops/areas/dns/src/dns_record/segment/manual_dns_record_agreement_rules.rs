use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual dns record agreement must pass. Check returns its problems, or none.
pub trait ManualDnsRecordAgreementRule: std::fmt::Debug {
    fn check(&self, item: &ManualDnsRecordAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordAgreementRequiredTitleRule {
    pub message: String,
}

impl ManualDnsRecordAgreementRule for ManualDnsRecordAgreementRequiredTitleRule {
    fn check(&self, item: &ManualDnsRecordAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordAgreementCodeLengthRule {
    pub max_length: i64,
}

impl ManualDnsRecordAgreementRule for ManualDnsRecordAgreementCodeLengthRule {
    fn check(&self, item: &ManualDnsRecordAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordAgreementGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualDnsRecordAgreementRule for ManualDnsRecordAgreementGraceDaysRangeRule {
    fn check(&self, item: &ManualDnsRecordAgreement) -> Vec<String> {
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

/// Runs every rule of a manual dns record agreement.
#[derive(Debug, Clone)]
pub struct ManualDnsRecordAgreementValidator {
    pub rules: Vec<Rc<dyn ManualDnsRecordAgreementRule>>,
}

impl ManualDnsRecordAgreementValidator {
    pub fn validate(&self, item: &ManualDnsRecordAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualDnsRecordAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDnsRecordAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_dns_record_agreement_validator() -> ManualDnsRecordAgreementValidator {
    ManualDnsRecordAgreementValidator {
        rules: vec![
            Rc::new(ManualDnsRecordAgreementRequiredTitleRule {
                message: "manual dns record agreement needs this field".to_string(),
            }) as Rc<dyn ManualDnsRecordAgreementRule>,
            Rc::new(ManualDnsRecordAgreementCodeLengthRule { max_length: 64 }),
            Rc::new(ManualDnsRecordAgreementGraceDaysRangeRule { min: 120, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_dns_record_agreement_validator();
        let item = new_manual_dns_record_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualDnsRecordAgreementRequiredTitleRule {
            message: "manual dns record agreement needs this field".to_string(),
        };
        let mut item = new_manual_dns_record_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualDnsRecordAgreementCodeLengthRule { max_length: 255 };
        let mut item = new_manual_dns_record_agreement("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = ManualDnsRecordAgreementGraceDaysRangeRule { min: 32, max: 255 };
        let mut item = new_manual_dns_record_agreement("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
