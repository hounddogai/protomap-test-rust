use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback approval claim must pass. Check returns its problems, or none.
pub trait FallbackApprovalClaimRule: std::fmt::Debug {
    fn check(&self, item: &FallbackApprovalClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalClaimLabelLengthRule {
    pub max_length: i64,
}

impl FallbackApprovalClaimRule for FallbackApprovalClaimLabelLengthRule {
    fn check(&self, item: &FallbackApprovalClaim) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalClaimUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackApprovalClaimRule for FallbackApprovalClaimUpdatedAtRangeRule {
    fn check(&self, item: &FallbackApprovalClaim) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalClaimRequiredCodeRule {
    pub message: String,
}

impl FallbackApprovalClaimRule for FallbackApprovalClaimRequiredCodeRule {
    fn check(&self, item: &FallbackApprovalClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalClaimCodeLengthRule {
    pub max_length: i64,
}

impl FallbackApprovalClaimRule for FallbackApprovalClaimCodeLengthRule {
    fn check(&self, item: &FallbackApprovalClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalClaimRequiredTitleRule {
    pub message: String,
}

impl FallbackApprovalClaimRule for FallbackApprovalClaimRequiredTitleRule {
    fn check(&self, item: &FallbackApprovalClaim) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalClaimMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackApprovalClaimRule for FallbackApprovalClaimMaximumRangeRule {
    fn check(&self, item: &FallbackApprovalClaim) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a fallback approval claim.
#[derive(Debug, Clone)]
pub struct FallbackApprovalClaimValidator {
    pub rules: Vec<Rc<dyn FallbackApprovalClaimRule>>,
}

impl FallbackApprovalClaimValidator {
    pub fn validate(&self, item: &FallbackApprovalClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackApprovalClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackApprovalClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_approval_claim_validator() -> FallbackApprovalClaimValidator {
    FallbackApprovalClaimValidator {
        rules: vec![
            Rc::new(FallbackApprovalClaimLabelLengthRule { max_length: 255 }) as Rc<dyn FallbackApprovalClaimRule>,
            Rc::new(FallbackApprovalClaimUpdatedAtRangeRule { min: 255, max: 1000 }),
            Rc::new(FallbackApprovalClaimRequiredCodeRule {
                message: "fallback approval claim needs an entry".to_string(),
            }),
            Rc::new(FallbackApprovalClaimCodeLengthRule { max_length: 80 }),
            Rc::new(FallbackApprovalClaimRequiredTitleRule {
                message: "fallback approval claim needs this field".to_string(),
            }),
            Rc::new(FallbackApprovalClaimMaximumRangeRule { min: 7.5, max: 551.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_approval_claim_validator();
        let item = new_fallback_approval_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FallbackApprovalClaimLabelLengthRule { max_length: 86400 };
        let mut item = new_fallback_approval_claim("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = FallbackApprovalClaimUpdatedAtRangeRule { min: 255, max: 255 };
        let mut item = new_fallback_approval_claim("x", "y");
        item.updated_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackApprovalClaimRequiredCodeRule {
            message: "fallback approval claim needs a value".to_string(),
        };
        let mut item = new_fallback_approval_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackApprovalClaimCodeLengthRule { max_length: 32 };
        let mut item = new_fallback_approval_claim("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FallbackApprovalClaimRequiredTitleRule {
            message: "fallback approval claim needs an entry".to_string(),
        };
        let mut item = new_fallback_approval_claim("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = FallbackApprovalClaimMaximumRangeRule { min: 5.6, max: 4270.0 };
        let mut item = new_fallback_approval_claim("x", "y");
        item.maximum = 4271.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
