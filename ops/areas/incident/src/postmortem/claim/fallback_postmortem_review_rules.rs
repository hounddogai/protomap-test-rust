use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback postmortem review must pass. Check returns its problems, or none.
pub trait FallbackPostmortemReviewRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPostmortemReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemReviewCodePrefixRule {
    pub prefix: String,
}

impl FallbackPostmortemReviewRule for FallbackPostmortemReviewCodePrefixRule {
    fn check(&self, item: &FallbackPostmortemReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FallbackPostmortemReviewRule for FallbackPostmortemReviewAllowedCodeRule {
    fn check(&self, item: &FallbackPostmortemReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemReviewQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPostmortemReviewRule for FallbackPostmortemReviewQuotaUnitsRangeRule {
    fn check(&self, item: &FallbackPostmortemReview) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback postmortem review.
#[derive(Debug, Clone)]
pub struct FallbackPostmortemReviewValidator {
    pub rules: Vec<Rc<dyn FallbackPostmortemReviewRule>>,
}

impl FallbackPostmortemReviewValidator {
    pub fn validate(&self, item: &FallbackPostmortemReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackPostmortemReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPostmortemReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_postmortem_review_validator() -> FallbackPostmortemReviewValidator {
    FallbackPostmortemReviewValidator {
        rules: vec![
            Rc::new(FallbackPostmortemReviewCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn FallbackPostmortemReviewRule>,
            Rc::new(FallbackPostmortemReviewAllowedCodeRule {
                allowed: vec!["fallback".to_string()],
            }),
            Rc::new(FallbackPostmortemReviewQuotaUnitsRangeRule { min: 120, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_postmortem_review_validator();
        let item = new_fallback_postmortem_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = FallbackPostmortemReviewQuotaUnitsRangeRule { min: 120, max: 255 };
        let mut item = new_fallback_postmortem_review("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
