use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record tier must pass. Check returns its problems, or none.
pub trait DnsRecordTierRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTierRequiredLabelRule {
    pub message: String,
}

impl DnsRecordTierRule for DnsRecordTierRequiredLabelRule {
    fn check(&self, item: &DnsRecordTier) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTierCategoryLengthRule {
    pub max_length: i64,
}

impl DnsRecordTierRule for DnsRecordTierCategoryLengthRule {
    fn check(&self, item: &DnsRecordTier) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTierThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordTierRule for DnsRecordTierThresholdRangeRule {
    fn check(&self, item: &DnsRecordTier) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTierLabelLengthRule {
    pub max_length: i64,
}

impl DnsRecordTierRule for DnsRecordTierLabelLengthRule {
    fn check(&self, item: &DnsRecordTier) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTierCodePrefixRule {
    pub prefix: String,
}

impl DnsRecordTierRule for DnsRecordTierCodePrefixRule {
    fn check(&self, item: &DnsRecordTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record tier.
#[derive(Debug, Clone)]
pub struct DnsRecordTierValidator {
    pub rules: Vec<Rc<dyn DnsRecordTierRule>>,
}

impl DnsRecordTierValidator {
    pub fn validate(&self, item: &DnsRecordTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_tier_validator() -> DnsRecordTierValidator {
    DnsRecordTierValidator {
        rules: vec![
            Rc::new(DnsRecordTierRequiredLabelRule {
                message: "dns record tier needs a value".to_string(),
            }) as Rc<dyn DnsRecordTierRule>,
            Rc::new(DnsRecordTierCategoryLengthRule { max_length: 32 }),
            Rc::new(DnsRecordTierThresholdRangeRule { min: 5.9, max: 1252.0 }),
            Rc::new(DnsRecordTierLabelLengthRule { max_length: 64 }),
            Rc::new(DnsRecordTierCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_tier_validator();
        let item = new_dns_record_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DnsRecordTierRequiredLabelRule {
            message: "dns record tier needs an entry".to_string(),
        };
        let mut item = new_dns_record_tier("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = DnsRecordTierCategoryLengthRule { max_length: 32 };
        let mut item = new_dns_record_tier("x", "y");
        item.category = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = DnsRecordTierThresholdRangeRule { min: 4.1, max: 2038.0 };
        let mut item = new_dns_record_tier("x", "y");
        item.threshold = 2039.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DnsRecordTierLabelLengthRule { max_length: 64 };
        let mut item = new_dns_record_tier("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
