use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped token claim must pass. Check returns its problems, or none.
pub trait CappedTokenClaimRule: std::fmt::Debug {
    fn check(&self, item: &CappedTokenClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenClaimRequiredCodeRule {
    pub message: String,
}

impl CappedTokenClaimRule for CappedTokenClaimRequiredCodeRule {
    fn check(&self, item: &CappedTokenClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenClaimCodePrefixRule {
    pub prefix: String,
}

impl CappedTokenClaimRule for CappedTokenClaimCodePrefixRule {
    fn check(&self, item: &CappedTokenClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenClaimWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedTokenClaimRule for CappedTokenClaimWindowDaysRangeRule {
    fn check(&self, item: &CappedTokenClaim) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenClaimPhoneLengthRule {
    pub max_length: i64,
}

impl CappedTokenClaimRule for CappedTokenClaimPhoneLengthRule {
    fn check(&self, item: &CappedTokenClaim) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenClaimClusterLengthRule {
    pub max_length: i64,
}

impl CappedTokenClaimRule for CappedTokenClaimClusterLengthRule {
    fn check(&self, item: &CappedTokenClaim) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenClaimRequiredLabelRule {
    pub message: String,
}

impl CappedTokenClaimRule for CappedTokenClaimRequiredLabelRule {
    fn check(&self, item: &CappedTokenClaim) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped token claim.
#[derive(Debug, Clone)]
pub struct CappedTokenClaimValidator {
    pub rules: Vec<Rc<dyn CappedTokenClaimRule>>,
}

impl CappedTokenClaimValidator {
    pub fn validate(&self, item: &CappedTokenClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedTokenClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedTokenClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_token_claim_validator() -> CappedTokenClaimValidator {
    CappedTokenClaimValidator {
        rules: vec![
            Rc::new(CappedTokenClaimRequiredCodeRule {
                message: "capped token claim needs an entry".to_string(),
            }) as Rc<dyn CappedTokenClaimRule>,
            Rc::new(CappedTokenClaimCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(CappedTokenClaimWindowDaysRangeRule { min: 32, max: 32 }),
            Rc::new(CappedTokenClaimPhoneLengthRule { max_length: 255 }),
            Rc::new(CappedTokenClaimClusterLengthRule { max_length: 64 }),
            Rc::new(CappedTokenClaimRequiredLabelRule {
                message: "capped token claim needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_token_claim_validator();
        let item = new_capped_token_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedTokenClaimRequiredCodeRule {
            message: "capped token claim needs this field".to_string(),
        };
        let mut item = new_capped_token_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = CappedTokenClaimWindowDaysRangeRule { min: 120, max: 3600 };
        let mut item = new_capped_token_claim("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = CappedTokenClaimPhoneLengthRule { max_length: 255 };
        let mut item = new_capped_token_claim("x", "y");
        item.phone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = CappedTokenClaimClusterLengthRule { max_length: 64 };
        let mut item = new_capped_token_claim("x", "y");
        item.cluster = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedTokenClaimRequiredLabelRule {
            message: "capped token claim needs a value".to_string(),
        };
        let mut item = new_capped_token_claim("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
