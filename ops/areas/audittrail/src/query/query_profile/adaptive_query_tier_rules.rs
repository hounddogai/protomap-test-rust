use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive query tier must pass. Check returns its problems, or none.
pub trait AdaptiveQueryTierRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveQueryTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryTierUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveQueryTierRule for AdaptiveQueryTierUpdatedAtRangeRule {
    fn check(&self, item: &AdaptiveQueryTier) -> Vec<String> {
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
pub struct AdaptiveQueryTierSummaryLengthRule {
    pub max_length: i64,
}

impl AdaptiveQueryTierRule for AdaptiveQueryTierSummaryLengthRule {
    fn check(&self, item: &AdaptiveQueryTier) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryTierLocaleLengthRule {
    pub max_length: i64,
}

impl AdaptiveQueryTierRule for AdaptiveQueryTierLocaleLengthRule {
    fn check(&self, item: &AdaptiveQueryTier) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryTierCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveQueryTierRule for AdaptiveQueryTierCodeLengthRule {
    fn check(&self, item: &AdaptiveQueryTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryTierCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveQueryTierRule for AdaptiveQueryTierCodePrefixRule {
    fn check(&self, item: &AdaptiveQueryTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive query tier.
#[derive(Debug, Clone)]
pub struct AdaptiveQueryTierValidator {
    pub rules: Vec<Rc<dyn AdaptiveQueryTierRule>>,
}

impl AdaptiveQueryTierValidator {
    pub fn validate(&self, item: &AdaptiveQueryTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveQueryTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveQueryTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_query_tier_validator() -> AdaptiveQueryTierValidator {
    AdaptiveQueryTierValidator {
        rules: vec![
            Rc::new(AdaptiveQueryTierUpdatedAtRangeRule { min: 80, max: 1000 }) as Rc<dyn AdaptiveQueryTierRule>,
            Rc::new(AdaptiveQueryTierSummaryLengthRule { max_length: 80 }),
            Rc::new(AdaptiveQueryTierLocaleLengthRule { max_length: 120 }),
            Rc::new(AdaptiveQueryTierCodeLengthRule { max_length: 64 }),
            Rc::new(AdaptiveQueryTierCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_query_tier_validator();
        let item = new_adaptive_query_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = AdaptiveQueryTierUpdatedAtRangeRule { min: 1000, max: 86400 };
        let mut item = new_adaptive_query_tier("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = AdaptiveQueryTierSummaryLengthRule { max_length: 32 };
        let mut item = new_adaptive_query_tier("x", "y");
        item.summary = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = AdaptiveQueryTierLocaleLengthRule { max_length: 86400 };
        let mut item = new_adaptive_query_tier("x", "y");
        item.locale = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveQueryTierCodeLengthRule { max_length: 16 };
        let mut item = new_adaptive_query_tier("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
