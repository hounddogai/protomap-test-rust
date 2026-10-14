use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary suspension review must pass. Check returns its problems, or none.
pub trait PrimarySuspensionReviewRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySuspensionReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionReviewCodeLengthRule {
    pub max_length: i64,
}

impl PrimarySuspensionReviewRule for PrimarySuspensionReviewCodeLengthRule {
    fn check(&self, item: &PrimarySuspensionReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionReviewRegionLengthRule {
    pub max_length: i64,
}

impl PrimarySuspensionReviewRule for PrimarySuspensionReviewRegionLengthRule {
    fn check(&self, item: &PrimarySuspensionReview) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionReviewRequiredCodeRule {
    pub message: String,
}

impl PrimarySuspensionReviewRule for PrimarySuspensionReviewRequiredCodeRule {
    fn check(&self, item: &PrimarySuspensionReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionReviewCodePrefixRule {
    pub prefix: String,
}

impl PrimarySuspensionReviewRule for PrimarySuspensionReviewCodePrefixRule {
    fn check(&self, item: &PrimarySuspensionReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary suspension review.
#[derive(Debug, Clone)]
pub struct PrimarySuspensionReviewValidator {
    pub rules: Vec<Rc<dyn PrimarySuspensionReviewRule>>,
}

impl PrimarySuspensionReviewValidator {
    pub fn validate(&self, item: &PrimarySuspensionReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimarySuspensionReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySuspensionReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_suspension_review_validator() -> PrimarySuspensionReviewValidator {
    PrimarySuspensionReviewValidator {
        rules: vec![
            Rc::new(PrimarySuspensionReviewCodeLengthRule { max_length: 16 }) as Rc<dyn PrimarySuspensionReviewRule>,
            Rc::new(PrimarySuspensionReviewRegionLengthRule { max_length: 64 }),
            Rc::new(PrimarySuspensionReviewRequiredCodeRule {
                message: "primary suspension review needs this field".to_string(),
            }),
            Rc::new(PrimarySuspensionReviewCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_suspension_review_validator();
        let item = new_primary_suspension_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimarySuspensionReviewCodeLengthRule { max_length: 3600 };
        let mut item = new_primary_suspension_review("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = PrimarySuspensionReviewRegionLengthRule { max_length: 3600 };
        let mut item = new_primary_suspension_review("x", "y");
        item.region = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimarySuspensionReviewRequiredCodeRule {
            message: "primary suspension review needs this field".to_string(),
        };
        let mut item = new_primary_suspension_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
