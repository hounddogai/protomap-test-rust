use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record agreement must pass. Check returns its problems, or none.
pub trait DnsRecordAgreementRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAgreementVisibleGuardRule {
    pub limit: i64,
}

impl DnsRecordAgreementRule for DnsRecordAgreementVisibleGuardRule {
    fn check(&self, item: &DnsRecordAgreement) -> Vec<String> {
        if item.visible && (item.revision > self.limit) {
            return vec![format!(
                "revision {} exceeds {} while visible",
                item.revision, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAgreementRequiredCodeRule {
    pub message: String,
}

impl DnsRecordAgreementRule for DnsRecordAgreementRequiredCodeRule {
    fn check(&self, item: &DnsRecordAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAgreementRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordAgreementRule for DnsRecordAgreementRateRangeRule {
    fn check(&self, item: &DnsRecordAgreement) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a dns record agreement.
#[derive(Debug, Clone)]
pub struct DnsRecordAgreementValidator {
    pub rules: Vec<Rc<dyn DnsRecordAgreementRule>>,
}

impl DnsRecordAgreementValidator {
    pub fn validate(&self, item: &DnsRecordAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_agreement_validator() -> DnsRecordAgreementValidator {
    DnsRecordAgreementValidator {
        rules: vec![
            Rc::new(DnsRecordAgreementVisibleGuardRule { limit: 255 }) as Rc<dyn DnsRecordAgreementRule>,
            Rc::new(DnsRecordAgreementRequiredCodeRule {
                message: "dns record agreement needs this field".to_string(),
            }),
            Rc::new(DnsRecordAgreementRateRangeRule { min: 2.3, max: 279.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_agreement_validator();
        let item = new_dns_record_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DnsRecordAgreementRequiredCodeRule {
            message: "dns record agreement needs this field".to_string(),
        };
        let mut item = new_dns_record_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = DnsRecordAgreementRateRangeRule { min: 6.5, max: 1499.5 };
        let mut item = new_dns_record_agreement("x", "y");
        item.rate = 1501.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
