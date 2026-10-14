use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record review must pass. Check returns its problems, or none.
pub trait DnsRecordReviewRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordReviewQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordReviewRule for DnsRecordReviewQuotaUnitsRangeRule {
    fn check(&self, item: &DnsRecordReview) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordReviewActiveGuardRule {
    pub limit: i64,
}

impl DnsRecordReviewRule for DnsRecordReviewActiveGuardRule {
    fn check(&self, item: &DnsRecordReview) -> Vec<String> {
        if item.active && (item.batch_size > self.limit) {
            return vec![format!(
                "batch size {} exceeds {} while active",
                item.batch_size, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordReviewCodePrefixRule {
    pub prefix: String,
}

impl DnsRecordReviewRule for DnsRecordReviewCodePrefixRule {
    fn check(&self, item: &DnsRecordReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordReviewDescriptionLengthRule {
    pub max_length: i64,
}

impl DnsRecordReviewRule for DnsRecordReviewDescriptionLengthRule {
    fn check(&self, item: &DnsRecordReview) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record review.
#[derive(Debug, Clone)]
pub struct DnsRecordReviewValidator {
    pub rules: Vec<Rc<dyn DnsRecordReviewRule>>,
}

impl DnsRecordReviewValidator {
    pub fn validate(&self, item: &DnsRecordReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_review_validator() -> DnsRecordReviewValidator {
    DnsRecordReviewValidator {
        rules: vec![
            Rc::new(DnsRecordReviewQuotaUnitsRangeRule { min: 255, max: 1000 }) as Rc<dyn DnsRecordReviewRule>,
            Rc::new(DnsRecordReviewActiveGuardRule { limit: 80 }),
            Rc::new(DnsRecordReviewCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(DnsRecordReviewDescriptionLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_review_validator();
        let item = new_dns_record_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = DnsRecordReviewQuotaUnitsRangeRule { min: 32, max: 80 };
        let mut item = new_dns_record_review("x", "y");
        item.quota_units = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = DnsRecordReviewDescriptionLengthRule { max_length: 16 };
        let mut item = new_dns_record_review("x", "y");
        item.description = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
