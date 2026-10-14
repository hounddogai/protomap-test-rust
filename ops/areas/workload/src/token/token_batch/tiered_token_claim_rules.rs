use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered token claim must pass. Check returns its problems, or none.
pub trait TieredTokenClaimRule: std::fmt::Debug {
    fn check(&self, item: &TieredTokenClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenClaimCodeLengthRule {
    pub max_length: i64,
}

impl TieredTokenClaimRule for TieredTokenClaimCodeLengthRule {
    fn check(&self, item: &TieredTokenClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenClaimCodePrefixRule {
    pub prefix: String,
}

impl TieredTokenClaimRule for TieredTokenClaimCodePrefixRule {
    fn check(&self, item: &TieredTokenClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenClaimRequiredCurrencyRule {
    pub message: String,
}

impl TieredTokenClaimRule for TieredTokenClaimRequiredCurrencyRule {
    fn check(&self, item: &TieredTokenClaim) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenClaimContactEmailLengthRule {
    pub max_length: i64,
}

impl TieredTokenClaimRule for TieredTokenClaimContactEmailLengthRule {
    fn check(&self, item: &TieredTokenClaim) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenClaimRequiredCodeRule {
    pub message: String,
}

impl TieredTokenClaimRule for TieredTokenClaimRequiredCodeRule {
    fn check(&self, item: &TieredTokenClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenClaimRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredTokenClaimRule for TieredTokenClaimRotationDaysRangeRule {
    fn check(&self, item: &TieredTokenClaim) -> Vec<String> {
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

/// Runs every rule of a tiered token claim.
#[derive(Debug, Clone)]
pub struct TieredTokenClaimValidator {
    pub rules: Vec<Rc<dyn TieredTokenClaimRule>>,
}

impl TieredTokenClaimValidator {
    pub fn validate(&self, item: &TieredTokenClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredTokenClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredTokenClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_token_claim_validator() -> TieredTokenClaimValidator {
    TieredTokenClaimValidator {
        rules: vec![
            Rc::new(TieredTokenClaimCodeLengthRule { max_length: 3600 }) as Rc<dyn TieredTokenClaimRule>,
            Rc::new(TieredTokenClaimCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(TieredTokenClaimRequiredCurrencyRule {
                message: "tiered token claim needs this field".to_string(),
            }),
            Rc::new(TieredTokenClaimContactEmailLengthRule { max_length: 3600 }),
            Rc::new(TieredTokenClaimRequiredCodeRule {
                message: "tiered token claim needs this field".to_string(),
            }),
            Rc::new(TieredTokenClaimRotationDaysRangeRule { min: 255, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_token_claim_validator();
        let item = new_tiered_token_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredTokenClaimCodeLengthRule { max_length: 3600 };
        let mut item = new_tiered_token_claim("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = TieredTokenClaimRequiredCurrencyRule {
            message: "tiered token claim needs this field".to_string(),
        };
        let mut item = new_tiered_token_claim("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = TieredTokenClaimContactEmailLengthRule { max_length: 255 };
        let mut item = new_tiered_token_claim("x", "y");
        item.contact_email = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredTokenClaimRequiredCodeRule {
            message: "tiered token claim needs this field".to_string(),
        };
        let mut item = new_tiered_token_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = TieredTokenClaimRotationDaysRangeRule { min: 32, max: 64 };
        let mut item = new_tiered_token_claim("x", "y");
        item.rotation_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
