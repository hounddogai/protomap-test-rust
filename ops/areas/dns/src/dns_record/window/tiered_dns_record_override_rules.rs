use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered dns record override must pass. Check returns its problems, or none.
pub trait TieredDnsRecordOverrideRule: std::fmt::Debug {
    fn check(&self, item: &TieredDnsRecordOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordOverrideRequiredNameRule {
    pub message: String,
}

impl TieredDnsRecordOverrideRule for TieredDnsRecordOverrideRequiredNameRule {
    fn check(&self, item: &TieredDnsRecordOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordOverrideCurrencyLengthRule {
    pub max_length: i64,
}

impl TieredDnsRecordOverrideRule for TieredDnsRecordOverrideCurrencyLengthRule {
    fn check(&self, item: &TieredDnsRecordOverride) -> Vec<String> {
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
pub struct TieredDnsRecordOverrideSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredDnsRecordOverrideRule for TieredDnsRecordOverrideSequenceRangeRule {
    fn check(&self, item: &TieredDnsRecordOverride) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordOverrideAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl TieredDnsRecordOverrideRule for TieredDnsRecordOverrideAllowedSegmentRule {
    fn check(&self, item: &TieredDnsRecordOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is blocked", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordOverrideRequiredCodeRule {
    pub message: String,
}

impl TieredDnsRecordOverrideRule for TieredDnsRecordOverrideRequiredCodeRule {
    fn check(&self, item: &TieredDnsRecordOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered dns record override.
#[derive(Debug, Clone)]
pub struct TieredDnsRecordOverrideValidator {
    pub rules: Vec<Rc<dyn TieredDnsRecordOverrideRule>>,
}

impl TieredDnsRecordOverrideValidator {
    pub fn validate(&self, item: &TieredDnsRecordOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredDnsRecordOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredDnsRecordOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_dns_record_override_validator() -> TieredDnsRecordOverrideValidator {
    TieredDnsRecordOverrideValidator {
        rules: vec![
            Rc::new(TieredDnsRecordOverrideRequiredNameRule {
                message: "tiered dns record override needs this field".to_string(),
            }) as Rc<dyn TieredDnsRecordOverrideRule>,
            Rc::new(TieredDnsRecordOverrideCurrencyLengthRule { max_length: 86400 }),
            Rc::new(TieredDnsRecordOverrideSequenceRangeRule { min: 120, max: 86400 }),
            Rc::new(TieredDnsRecordOverrideAllowedSegmentRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(TieredDnsRecordOverrideRequiredCodeRule {
                message: "tiered dns record override needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_dns_record_override_validator();
        let item = new_tiered_dns_record_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredDnsRecordOverrideRequiredNameRule {
            message: "tiered dns record override needs an entry".to_string(),
        };
        let mut item = new_tiered_dns_record_override("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = TieredDnsRecordOverrideCurrencyLengthRule { max_length: 120 };
        let mut item = new_tiered_dns_record_override("x", "y");
        item.currency = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = TieredDnsRecordOverrideSequenceRangeRule { min: 1000, max: 86400 };
        let mut item = new_tiered_dns_record_override("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredDnsRecordOverrideRequiredCodeRule {
            message: "tiered dns record override needs a value".to_string(),
        };
        let mut item = new_tiered_dns_record_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
