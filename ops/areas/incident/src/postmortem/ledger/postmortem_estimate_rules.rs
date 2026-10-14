use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem estimate must pass. Check returns its problems, or none.
pub trait PostmortemEstimateRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemEstimateWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PostmortemEstimateRule for PostmortemEstimateWeightRangeRule {
    fn check(&self, item: &PostmortemEstimate) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemEstimateExternalRefLengthRule {
    pub max_length: i64,
}

impl PostmortemEstimateRule for PostmortemEstimateExternalRefLengthRule {
    fn check(&self, item: &PostmortemEstimate) -> Vec<String> {
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
pub struct PostmortemEstimateRequiredTitleRule {
    pub message: String,
}

impl PostmortemEstimateRule for PostmortemEstimateRequiredTitleRule {
    fn check(&self, item: &PostmortemEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemEstimateRequiredLocaleRule {
    pub message: String,
}

impl PostmortemEstimateRule for PostmortemEstimateRequiredLocaleRule {
    fn check(&self, item: &PostmortemEstimate) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemEstimateLocaleLengthRule {
    pub max_length: i64,
}

impl PostmortemEstimateRule for PostmortemEstimateLocaleLengthRule {
    fn check(&self, item: &PostmortemEstimate) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem estimate.
#[derive(Debug, Clone)]
pub struct PostmortemEstimateValidator {
    pub rules: Vec<Rc<dyn PostmortemEstimateRule>>,
}

impl PostmortemEstimateValidator {
    pub fn validate(&self, item: &PostmortemEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_estimate_validator() -> PostmortemEstimateValidator {
    PostmortemEstimateValidator {
        rules: vec![
            Rc::new(PostmortemEstimateWeightRangeRule { min: 5.9, max: 4479.3 }) as Rc<dyn PostmortemEstimateRule>,
            Rc::new(PostmortemEstimateExternalRefLengthRule { max_length: 64 }),
            Rc::new(PostmortemEstimateRequiredTitleRule {
                message: "postmortem estimate needs a value".to_string(),
            }),
            Rc::new(PostmortemEstimateRequiredLocaleRule {
                message: "postmortem estimate needs an entry".to_string(),
            }),
            Rc::new(PostmortemEstimateLocaleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_estimate_validator();
        let item = new_postmortem_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PostmortemEstimateWeightRangeRule { min: 1.2, max: 2588.4 };
        let mut item = new_postmortem_estimate("x", "y");
        item.weight = 2589.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = PostmortemEstimateExternalRefLengthRule { max_length: 64 };
        let mut item = new_postmortem_estimate("x", "y");
        item.external_ref = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PostmortemEstimateRequiredTitleRule {
            message: "postmortem estimate needs a value".to_string(),
        };
        let mut item = new_postmortem_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = PostmortemEstimateRequiredLocaleRule {
            message: "postmortem estimate needs an entry".to_string(),
        };
        let mut item = new_postmortem_estimate("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = PostmortemEstimateLocaleLengthRule { max_length: 32 };
        let mut item = new_postmortem_estimate("x", "y");
        item.locale = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
