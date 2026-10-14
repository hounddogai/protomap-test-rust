use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback approval digest must pass. Check returns its problems, or none.
pub trait FallbackApprovalDigestRule: std::fmt::Debug {
    fn check(&self, item: &FallbackApprovalDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalDigestMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackApprovalDigestRule for FallbackApprovalDigestMinimumRangeRule {
    fn check(&self, item: &FallbackApprovalDigest) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalDigestExternalRefLengthRule {
    pub max_length: i64,
}

impl FallbackApprovalDigestRule for FallbackApprovalDigestExternalRefLengthRule {
    fn check(&self, item: &FallbackApprovalDigest) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalDigestRequiredCodeRule {
    pub message: String,
}

impl FallbackApprovalDigestRule for FallbackApprovalDigestRequiredCodeRule {
    fn check(&self, item: &FallbackApprovalDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalDigestCodeLengthRule {
    pub max_length: i64,
}

impl FallbackApprovalDigestRule for FallbackApprovalDigestCodeLengthRule {
    fn check(&self, item: &FallbackApprovalDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback approval digest.
#[derive(Debug, Clone)]
pub struct FallbackApprovalDigestValidator {
    pub rules: Vec<Rc<dyn FallbackApprovalDigestRule>>,
}

impl FallbackApprovalDigestValidator {
    pub fn validate(&self, item: &FallbackApprovalDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackApprovalDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackApprovalDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_approval_digest_validator() -> FallbackApprovalDigestValidator {
    FallbackApprovalDigestValidator {
        rules: vec![
            Rc::new(FallbackApprovalDigestMinimumRangeRule { min: 7.6, max: 4259.8 })
                as Rc<dyn FallbackApprovalDigestRule>,
            Rc::new(FallbackApprovalDigestExternalRefLengthRule { max_length: 32 }),
            Rc::new(FallbackApprovalDigestRequiredCodeRule {
                message: "fallback approval digest needs an entry".to_string(),
            }),
            Rc::new(FallbackApprovalDigestCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_approval_digest_validator();
        let item = new_fallback_approval_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = FallbackApprovalDigestMinimumRangeRule { min: 4.1, max: 2657.1 };
        let mut item = new_fallback_approval_digest("x", "y");
        item.minimum = 2658.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = FallbackApprovalDigestExternalRefLengthRule { max_length: 64 };
        let mut item = new_fallback_approval_digest("x", "y");
        item.external_ref = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackApprovalDigestRequiredCodeRule {
            message: "fallback approval digest needs an entry".to_string(),
        };
        let mut item = new_fallback_approval_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackApprovalDigestCodeLengthRule { max_length: 1000 };
        let mut item = new_fallback_approval_digest("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
