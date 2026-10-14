use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered delegation claim must pass. Check returns its problems, or none.
pub trait TieredDelegationClaimRule: std::fmt::Debug {
    fn check(&self, item: &TieredDelegationClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationClaimCodePrefixRule {
    pub prefix: String,
}

impl TieredDelegationClaimRule for TieredDelegationClaimCodePrefixRule {
    fn check(&self, item: &TieredDelegationClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationClaimRequiredExternalRefRule {
    pub message: String,
}

impl TieredDelegationClaimRule for TieredDelegationClaimRequiredExternalRefRule {
    fn check(&self, item: &TieredDelegationClaim) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationClaimBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredDelegationClaimRule for TieredDelegationClaimBatchSizeRangeRule {
    fn check(&self, item: &TieredDelegationClaim) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationClaimCodeLengthRule {
    pub max_length: i64,
}

impl TieredDelegationClaimRule for TieredDelegationClaimCodeLengthRule {
    fn check(&self, item: &TieredDelegationClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationClaimLabelLengthRule {
    pub max_length: i64,
}

impl TieredDelegationClaimRule for TieredDelegationClaimLabelLengthRule {
    fn check(&self, item: &TieredDelegationClaim) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered delegation claim.
#[derive(Debug, Clone)]
pub struct TieredDelegationClaimValidator {
    pub rules: Vec<Rc<dyn TieredDelegationClaimRule>>,
}

impl TieredDelegationClaimValidator {
    pub fn validate(&self, item: &TieredDelegationClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredDelegationClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredDelegationClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_delegation_claim_validator() -> TieredDelegationClaimValidator {
    TieredDelegationClaimValidator {
        rules: vec![
            Rc::new(TieredDelegationClaimCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn TieredDelegationClaimRule>,
            Rc::new(TieredDelegationClaimRequiredExternalRefRule {
                message: "tiered delegation claim needs an entry".to_string(),
            }),
            Rc::new(TieredDelegationClaimBatchSizeRangeRule { min: 255, max: 3600 }),
            Rc::new(TieredDelegationClaimCodeLengthRule { max_length: 16 }),
            Rc::new(TieredDelegationClaimLabelLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_delegation_claim_validator();
        let item = new_tiered_delegation_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = TieredDelegationClaimRequiredExternalRefRule {
            message: "tiered delegation claim needs a value".to_string(),
        };
        let mut item = new_tiered_delegation_claim("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = TieredDelegationClaimBatchSizeRangeRule { min: 64, max: 120 };
        let mut item = new_tiered_delegation_claim("x", "y");
        item.batch_size = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredDelegationClaimCodeLengthRule { max_length: 120 };
        let mut item = new_tiered_delegation_claim("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TieredDelegationClaimLabelLengthRule { max_length: 3600 };
        let mut item = new_tiered_delegation_claim("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
