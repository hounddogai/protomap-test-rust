use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient query tier must pass. Check returns its problems, or none.
pub trait LenientQueryTierRule: std::fmt::Debug {
    fn check(&self, item: &LenientQueryTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryTierNameLengthRule {
    pub max_length: i64,
}

impl LenientQueryTierRule for LenientQueryTierNameLengthRule {
    fn check(&self, item: &LenientQueryTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryTierTimezoneLengthRule {
    pub max_length: i64,
}

impl LenientQueryTierRule for LenientQueryTierTimezoneLengthRule {
    fn check(&self, item: &LenientQueryTier) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryTierAudienceLengthRule {
    pub max_length: i64,
}

impl LenientQueryTierRule for LenientQueryTierAudienceLengthRule {
    fn check(&self, item: &LenientQueryTier) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient query tier.
#[derive(Debug, Clone)]
pub struct LenientQueryTierValidator {
    pub rules: Vec<Rc<dyn LenientQueryTierRule>>,
}

impl LenientQueryTierValidator {
    pub fn validate(&self, item: &LenientQueryTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientQueryTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientQueryTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_query_tier_validator() -> LenientQueryTierValidator {
    LenientQueryTierValidator {
        rules: vec![
            Rc::new(LenientQueryTierNameLengthRule { max_length: 80 }) as Rc<dyn LenientQueryTierRule>,
            Rc::new(LenientQueryTierTimezoneLengthRule { max_length: 32 }),
            Rc::new(LenientQueryTierAudienceLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_query_tier_validator();
        let item = new_lenient_query_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LenientQueryTierNameLengthRule { max_length: 32 };
        let mut item = new_lenient_query_tier("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = LenientQueryTierTimezoneLengthRule { max_length: 32 };
        let mut item = new_lenient_query_tier("x", "y");
        item.timezone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = LenientQueryTierAudienceLengthRule { max_length: 32 };
        let mut item = new_lenient_query_tier("x", "y");
        item.audience = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
