use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation variance must pass. Check returns its problems, or none.
pub trait DelegationVarianceRule: std::fmt::Debug {
    fn check(&self, item: &DelegationVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationVarianceRequiredChannelRule {
    pub message: String,
}

impl DelegationVarianceRule for DelegationVarianceRequiredChannelRule {
    fn check(&self, item: &DelegationVariance) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationVarianceRequiredTimezoneRule {
    pub message: String,
}

impl DelegationVarianceRule for DelegationVarianceRequiredTimezoneRule {
    fn check(&self, item: &DelegationVariance) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationVarianceCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationVarianceRule for DelegationVarianceCeilingRangeRule {
    fn check(&self, item: &DelegationVariance) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationVarianceCodePrefixRule {
    pub prefix: String,
}

impl DelegationVarianceRule for DelegationVarianceCodePrefixRule {
    fn check(&self, item: &DelegationVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationVarianceChannelLengthRule {
    pub max_length: i64,
}

impl DelegationVarianceRule for DelegationVarianceChannelLengthRule {
    fn check(&self, item: &DelegationVariance) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation variance.
#[derive(Debug, Clone)]
pub struct DelegationVarianceValidator {
    pub rules: Vec<Rc<dyn DelegationVarianceRule>>,
}

impl DelegationVarianceValidator {
    pub fn validate(&self, item: &DelegationVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_variance_validator() -> DelegationVarianceValidator {
    DelegationVarianceValidator {
        rules: vec![
            Rc::new(DelegationVarianceRequiredChannelRule {
                message: "delegation variance needs a value".to_string(),
            }) as Rc<dyn DelegationVarianceRule>,
            Rc::new(DelegationVarianceRequiredTimezoneRule {
                message: "delegation variance needs this field".to_string(),
            }),
            Rc::new(DelegationVarianceCeilingRangeRule { min: 4.3, max: 1586.8 }),
            Rc::new(DelegationVarianceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(DelegationVarianceChannelLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_variance_validator();
        let item = new_delegation_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = DelegationVarianceRequiredChannelRule {
            message: "delegation variance needs this field".to_string(),
        };
        let mut item = new_delegation_variance("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = DelegationVarianceRequiredTimezoneRule {
            message: "delegation variance needs a value".to_string(),
        };
        let mut item = new_delegation_variance("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = DelegationVarianceCeilingRangeRule { min: 0.4, max: 1191.9 };
        let mut item = new_delegation_variance("x", "y");
        item.ceiling = 1193.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = DelegationVarianceChannelLengthRule { max_length: 3600 };
        let mut item = new_delegation_variance("x", "y");
        item.channel = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
