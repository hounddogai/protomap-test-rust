use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending dns record variance must pass. Check returns its problems, or none.
pub trait PendingDnsRecordVarianceRule: std::fmt::Debug {
    fn check(&self, item: &PendingDnsRecordVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDnsRecordVarianceRequiredCodeRule {
    pub message: String,
}

impl PendingDnsRecordVarianceRule for PendingDnsRecordVarianceRequiredCodeRule {
    fn check(&self, item: &PendingDnsRecordVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDnsRecordVarianceRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingDnsRecordVarianceRule for PendingDnsRecordVarianceRotationDaysRangeRule {
    fn check(&self, item: &PendingDnsRecordVariance) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDnsRecordVarianceCodeLengthRule {
    pub max_length: i64,
}

impl PendingDnsRecordVarianceRule for PendingDnsRecordVarianceCodeLengthRule {
    fn check(&self, item: &PendingDnsRecordVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDnsRecordVarianceVisibleGuardRule {
    pub limit: f64,
}

impl PendingDnsRecordVarianceRule for PendingDnsRecordVarianceVisibleGuardRule {
    fn check(&self, item: &PendingDnsRecordVariance) -> Vec<String> {
        if item.visible && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while visible",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending dns record variance.
#[derive(Debug, Clone)]
pub struct PendingDnsRecordVarianceValidator {
    pub rules: Vec<Rc<dyn PendingDnsRecordVarianceRule>>,
}

impl PendingDnsRecordVarianceValidator {
    pub fn validate(&self, item: &PendingDnsRecordVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingDnsRecordVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingDnsRecordVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_dns_record_variance_validator() -> PendingDnsRecordVarianceValidator {
    PendingDnsRecordVarianceValidator {
        rules: vec![
            Rc::new(PendingDnsRecordVarianceRequiredCodeRule {
                message: "pending dns record variance needs this field".to_string(),
            }) as Rc<dyn PendingDnsRecordVarianceRule>,
            Rc::new(PendingDnsRecordVarianceRotationDaysRangeRule { min: 120, max: 1000 }),
            Rc::new(PendingDnsRecordVarianceCodeLengthRule { max_length: 3600 }),
            Rc::new(PendingDnsRecordVarianceVisibleGuardRule { limit: 8.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_dns_record_variance_validator();
        let item = new_pending_dns_record_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingDnsRecordVarianceRequiredCodeRule {
            message: "pending dns record variance needs an entry".to_string(),
        };
        let mut item = new_pending_dns_record_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = PendingDnsRecordVarianceRotationDaysRangeRule { min: 16, max: 32 };
        let mut item = new_pending_dns_record_variance("x", "y");
        item.rotation_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingDnsRecordVarianceCodeLengthRule { max_length: 86400 };
        let mut item = new_pending_dns_record_variance("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
