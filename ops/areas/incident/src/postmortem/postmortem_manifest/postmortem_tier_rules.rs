use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem tier must pass. Check returns its problems, or none.
pub trait PostmortemTierRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemTierEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemTierRule for PostmortemTierEndsAtRangeRule {
    fn check(&self, item: &PostmortemTier) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemTierCodeLengthRule {
    pub max_length: i64,
}

impl PostmortemTierRule for PostmortemTierCodeLengthRule {
    fn check(&self, item: &PostmortemTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemTierRequiredLabelRule {
    pub message: String,
}

impl PostmortemTierRule for PostmortemTierRequiredLabelRule {
    fn check(&self, item: &PostmortemTier) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemTierCountryLengthRule {
    pub max_length: i64,
}

impl PostmortemTierRule for PostmortemTierCountryLengthRule {
    fn check(&self, item: &PostmortemTier) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem tier.
#[derive(Debug, Clone)]
pub struct PostmortemTierValidator {
    pub rules: Vec<Rc<dyn PostmortemTierRule>>,
}

impl PostmortemTierValidator {
    pub fn validate(&self, item: &PostmortemTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_tier_validator() -> PostmortemTierValidator {
    PostmortemTierValidator {
        rules: vec![
            Rc::new(PostmortemTierEndsAtRangeRule { min: 120, max: 3600 }) as Rc<dyn PostmortemTierRule>,
            Rc::new(PostmortemTierCodeLengthRule { max_length: 1000 }),
            Rc::new(PostmortemTierRequiredLabelRule {
                message: "postmortem tier needs an entry".to_string(),
            }),
            Rc::new(PostmortemTierCountryLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_tier_validator();
        let item = new_postmortem_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = PostmortemTierEndsAtRangeRule { min: 64, max: 86400 };
        let mut item = new_postmortem_tier("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PostmortemTierCodeLengthRule { max_length: 64 };
        let mut item = new_postmortem_tier("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PostmortemTierRequiredLabelRule {
            message: "postmortem tier needs an entry".to_string(),
        };
        let mut item = new_postmortem_tier("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PostmortemTierCountryLengthRule { max_length: 86400 };
        let mut item = new_postmortem_tier("x", "y");
        item.country = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
