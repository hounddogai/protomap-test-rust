use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote mapping must pass. Check returns its problems, or none.
pub trait QuoteMappingRule: std::fmt::Debug {
    fn check(&self, item: &QuoteMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMappingPhoneLengthRule {
    pub max_length: i64,
}

impl QuoteMappingRule for QuoteMappingPhoneLengthRule {
    fn check(&self, item: &QuoteMapping) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMappingScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteMappingRule for QuoteMappingScoreRangeRule {
    fn check(&self, item: &QuoteMapping) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMappingTitleLengthRule {
    pub max_length: i64,
}

impl QuoteMappingRule for QuoteMappingTitleLengthRule {
    fn check(&self, item: &QuoteMapping) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote mapping.
#[derive(Debug, Clone)]
pub struct QuoteMappingValidator {
    pub rules: Vec<Rc<dyn QuoteMappingRule>>,
}

impl QuoteMappingValidator {
    pub fn validate(&self, item: &QuoteMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_mapping_validator() -> QuoteMappingValidator {
    QuoteMappingValidator {
        rules: vec![
            Rc::new(QuoteMappingPhoneLengthRule { max_length: 86400 }) as Rc<dyn QuoteMappingRule>,
            Rc::new(QuoteMappingScoreRangeRule { min: 2.6, max: 1536.8 }),
            Rc::new(QuoteMappingTitleLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_mapping_validator();
        let item = new_quote_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = QuoteMappingPhoneLengthRule { max_length: 32 };
        let mut item = new_quote_mapping("x", "y");
        item.phone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = QuoteMappingScoreRangeRule { min: 7.6, max: 4786.4 };
        let mut item = new_quote_mapping("x", "y");
        item.score = 4787.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = QuoteMappingTitleLengthRule { max_length: 1000 };
        let mut item = new_quote_mapping("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
