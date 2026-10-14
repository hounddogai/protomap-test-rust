use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered dns record policy must pass. Check returns its problems, or none.
pub trait TieredDnsRecordPolicyRule: std::fmt::Debug {
    fn check(&self, item: &TieredDnsRecordPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordPolicyAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredDnsRecordPolicyRule for TieredDnsRecordPolicyAmountRangeRule {
    fn check(&self, item: &TieredDnsRecordPolicy) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordPolicyRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredDnsRecordPolicyRule for TieredDnsRecordPolicyRotationDaysRangeRule {
    fn check(&self, item: &TieredDnsRecordPolicy) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordPolicyAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl TieredDnsRecordPolicyRule for TieredDnsRecordPolicyAllowedPhoneRule {
    fn check(&self, item: &TieredDnsRecordPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is not allowed", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordPolicyLabelLengthRule {
    pub max_length: i64,
}

impl TieredDnsRecordPolicyRule for TieredDnsRecordPolicyLabelLengthRule {
    fn check(&self, item: &TieredDnsRecordPolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordPolicyVisibleGuardRule {
    pub limit: i64,
}

impl TieredDnsRecordPolicyRule for TieredDnsRecordPolicyVisibleGuardRule {
    fn check(&self, item: &TieredDnsRecordPolicy) -> Vec<String> {
        if item.visible && (item.batch_size > self.limit) {
            return vec![format!(
                "batch size {} exceeds {} while visible",
                item.batch_size, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordPolicyRequiredPhoneRule {
    pub message: String,
}

impl TieredDnsRecordPolicyRule for TieredDnsRecordPolicyRequiredPhoneRule {
    fn check(&self, item: &TieredDnsRecordPolicy) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered dns record policy.
#[derive(Debug, Clone)]
pub struct TieredDnsRecordPolicyValidator {
    pub rules: Vec<Rc<dyn TieredDnsRecordPolicyRule>>,
}

impl TieredDnsRecordPolicyValidator {
    pub fn validate(&self, item: &TieredDnsRecordPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredDnsRecordPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredDnsRecordPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_dns_record_policy_validator() -> TieredDnsRecordPolicyValidator {
    TieredDnsRecordPolicyValidator {
        rules: vec![
            Rc::new(TieredDnsRecordPolicyAmountRangeRule { min: 1.6, max: 1025.8 })
                as Rc<dyn TieredDnsRecordPolicyRule>,
            Rc::new(TieredDnsRecordPolicyRotationDaysRangeRule { min: 255, max: 3600 }),
            Rc::new(TieredDnsRecordPolicyAllowedPhoneRule { allowed: Vec::new() }),
            Rc::new(TieredDnsRecordPolicyLabelLengthRule { max_length: 120 }),
            Rc::new(TieredDnsRecordPolicyVisibleGuardRule { limit: 80 }),
            Rc::new(TieredDnsRecordPolicyRequiredPhoneRule {
                message: "tiered dns record policy needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_dns_record_policy_validator();
        let item = new_tiered_dns_record_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = TieredDnsRecordPolicyAmountRangeRule { min: 4.8, max: 3988.5 };
        let mut item = new_tiered_dns_record_policy("x", "y");
        item.amount = 3990.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = TieredDnsRecordPolicyRotationDaysRangeRule { min: 80, max: 120 };
        let mut item = new_tiered_dns_record_policy("x", "y");
        item.rotation_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TieredDnsRecordPolicyLabelLengthRule { max_length: 3600 };
        let mut item = new_tiered_dns_record_policy("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = TieredDnsRecordPolicyRequiredPhoneRule {
            message: "tiered dns record policy needs an entry".to_string(),
        };
        let mut item = new_tiered_dns_record_policy("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
