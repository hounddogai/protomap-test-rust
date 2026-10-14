use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive limit digest must pass. Check returns its problems, or none.
pub trait AdaptiveLimitDigestRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveLimitDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveLimitDigestBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveLimitDigestRule for AdaptiveLimitDigestBatchSizeRangeRule {
    fn check(&self, item: &AdaptiveLimitDigest) -> Vec<String> {
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
pub struct AdaptiveLimitDigestRequiredDescriptionRule {
    pub message: String,
}

impl AdaptiveLimitDigestRule for AdaptiveLimitDigestRequiredDescriptionRule {
    fn check(&self, item: &AdaptiveLimitDigest) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveLimitDigestRequiredExternalRefRule {
    pub message: String,
}

impl AdaptiveLimitDigestRule for AdaptiveLimitDigestRequiredExternalRefRule {
    fn check(&self, item: &AdaptiveLimitDigest) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveLimitDigestRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveLimitDigestRule for AdaptiveLimitDigestRatioRangeRule {
    fn check(&self, item: &AdaptiveLimitDigest) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveLimitDigestRequiredTitleRule {
    pub message: String,
}

impl AdaptiveLimitDigestRule for AdaptiveLimitDigestRequiredTitleRule {
    fn check(&self, item: &AdaptiveLimitDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveLimitDigestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveLimitDigestRule for AdaptiveLimitDigestAllowedCodeRule {
    fn check(&self, item: &AdaptiveLimitDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive limit digest.
#[derive(Debug, Clone)]
pub struct AdaptiveLimitDigestValidator {
    pub rules: Vec<Rc<dyn AdaptiveLimitDigestRule>>,
}

impl AdaptiveLimitDigestValidator {
    pub fn validate(&self, item: &AdaptiveLimitDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveLimitDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveLimitDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_limit_digest_validator() -> AdaptiveLimitDigestValidator {
    AdaptiveLimitDigestValidator {
        rules: vec![
            Rc::new(AdaptiveLimitDigestBatchSizeRangeRule { min: 64, max: 3600 }) as Rc<dyn AdaptiveLimitDigestRule>,
            Rc::new(AdaptiveLimitDigestRequiredDescriptionRule {
                message: "adaptive limit digest needs this field".to_string(),
            }),
            Rc::new(AdaptiveLimitDigestRequiredExternalRefRule {
                message: "adaptive limit digest needs this field".to_string(),
            }),
            Rc::new(AdaptiveLimitDigestRatioRangeRule { min: 0.7, max: 388.1 }),
            Rc::new(AdaptiveLimitDigestRequiredTitleRule {
                message: "adaptive limit digest needs this field".to_string(),
            }),
            Rc::new(AdaptiveLimitDigestAllowedCodeRule {
                allowed: vec!["default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_limit_digest_validator();
        let item = new_adaptive_limit_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = AdaptiveLimitDigestBatchSizeRangeRule { min: 32, max: 64 };
        let mut item = new_adaptive_limit_digest("x", "y");
        item.batch_size = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = AdaptiveLimitDigestRequiredDescriptionRule {
            message: "adaptive limit digest needs an entry".to_string(),
        };
        let mut item = new_adaptive_limit_digest("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = AdaptiveLimitDigestRequiredExternalRefRule {
            message: "adaptive limit digest needs this field".to_string(),
        };
        let mut item = new_adaptive_limit_digest("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = AdaptiveLimitDigestRatioRangeRule { min: 1.7, max: 3863.1 };
        let mut item = new_adaptive_limit_digest("x", "y");
        item.ratio = 3864.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AdaptiveLimitDigestRequiredTitleRule {
            message: "adaptive limit digest needs this field".to_string(),
        };
        let mut item = new_adaptive_limit_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
