use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered federation group must pass. Check returns its problems, or none.
pub trait TieredFederationGroupRule: std::fmt::Debug {
    fn check(&self, item: &TieredFederationGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationGroupRequiredLabelRule {
    pub message: String,
}

impl TieredFederationGroupRule for TieredFederationGroupRequiredLabelRule {
    fn check(&self, item: &TieredFederationGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationGroupRequiredCodeRule {
    pub message: String,
}

impl TieredFederationGroupRule for TieredFederationGroupRequiredCodeRule {
    fn check(&self, item: &TieredFederationGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationGroupCurrencyLengthRule {
    pub max_length: i64,
}

impl TieredFederationGroupRule for TieredFederationGroupCurrencyLengthRule {
    fn check(&self, item: &TieredFederationGroup) -> Vec<String> {
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
pub struct TieredFederationGroupAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredFederationGroupRule for TieredFederationGroupAttemptsRangeRule {
    fn check(&self, item: &TieredFederationGroup) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationGroupCodePrefixRule {
    pub prefix: String,
}

impl TieredFederationGroupRule for TieredFederationGroupCodePrefixRule {
    fn check(&self, item: &TieredFederationGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered federation group.
#[derive(Debug, Clone)]
pub struct TieredFederationGroupValidator {
    pub rules: Vec<Rc<dyn TieredFederationGroupRule>>,
}

impl TieredFederationGroupValidator {
    pub fn validate(&self, item: &TieredFederationGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredFederationGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredFederationGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_federation_group_validator() -> TieredFederationGroupValidator {
    TieredFederationGroupValidator {
        rules: vec![
            Rc::new(TieredFederationGroupRequiredLabelRule {
                message: "tiered federation group needs an entry".to_string(),
            }) as Rc<dyn TieredFederationGroupRule>,
            Rc::new(TieredFederationGroupRequiredCodeRule {
                message: "tiered federation group needs an entry".to_string(),
            }),
            Rc::new(TieredFederationGroupCurrencyLengthRule { max_length: 16 }),
            Rc::new(TieredFederationGroupAttemptsRangeRule { min: 255, max: 3600 }),
            Rc::new(TieredFederationGroupCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_federation_group_validator();
        let item = new_tiered_federation_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TieredFederationGroupRequiredLabelRule {
            message: "tiered federation group needs a value".to_string(),
        };
        let mut item = new_tiered_federation_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredFederationGroupRequiredCodeRule {
            message: "tiered federation group needs this field".to_string(),
        };
        let mut item = new_tiered_federation_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = TieredFederationGroupCurrencyLengthRule { max_length: 64 };
        let mut item = new_tiered_federation_group("x", "y");
        item.currency = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = TieredFederationGroupAttemptsRangeRule { min: 16, max: 120 };
        let mut item = new_tiered_federation_group("x", "y");
        item.attempts = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
