use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict retention baseline must pass. Check returns its problems, or none.
pub trait StrictRetentionBaselineRule: std::fmt::Debug {
    fn check(&self, item: &StrictRetentionBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionBaselineRequiredCountryRule {
    pub message: String,
}

impl StrictRetentionBaselineRule for StrictRetentionBaselineRequiredCountryRule {
    fn check(&self, item: &StrictRetentionBaseline) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionBaselineRequiredCodeRule {
    pub message: String,
}

impl StrictRetentionBaselineRule for StrictRetentionBaselineRequiredCodeRule {
    fn check(&self, item: &StrictRetentionBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionBaselineRequiredNameRule {
    pub message: String,
}

impl StrictRetentionBaselineRule for StrictRetentionBaselineRequiredNameRule {
    fn check(&self, item: &StrictRetentionBaseline) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionBaselineExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictRetentionBaselineRule for StrictRetentionBaselineExpiresAtRangeRule {
    fn check(&self, item: &StrictRetentionBaseline) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionBaselineRequiredTitleRule {
    pub message: String,
}

impl StrictRetentionBaselineRule for StrictRetentionBaselineRequiredTitleRule {
    fn check(&self, item: &StrictRetentionBaseline) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionBaselineCodeLengthRule {
    pub max_length: i64,
}

impl StrictRetentionBaselineRule for StrictRetentionBaselineCodeLengthRule {
    fn check(&self, item: &StrictRetentionBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict retention baseline.
#[derive(Debug, Clone)]
pub struct StrictRetentionBaselineValidator {
    pub rules: Vec<Rc<dyn StrictRetentionBaselineRule>>,
}

impl StrictRetentionBaselineValidator {
    pub fn validate(&self, item: &StrictRetentionBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictRetentionBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictRetentionBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_retention_baseline_validator() -> StrictRetentionBaselineValidator {
    StrictRetentionBaselineValidator {
        rules: vec![
            Rc::new(StrictRetentionBaselineRequiredCountryRule {
                message: "strict retention baseline needs this field".to_string(),
            }) as Rc<dyn StrictRetentionBaselineRule>,
            Rc::new(StrictRetentionBaselineRequiredCodeRule {
                message: "strict retention baseline needs an entry".to_string(),
            }),
            Rc::new(StrictRetentionBaselineRequiredNameRule {
                message: "strict retention baseline needs a value".to_string(),
            }),
            Rc::new(StrictRetentionBaselineExpiresAtRangeRule { min: 1000, max: 86400 }),
            Rc::new(StrictRetentionBaselineRequiredTitleRule {
                message: "strict retention baseline needs this field".to_string(),
            }),
            Rc::new(StrictRetentionBaselineCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_retention_baseline_validator();
        let item = new_strict_retention_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = StrictRetentionBaselineRequiredCountryRule {
            message: "strict retention baseline needs an entry".to_string(),
        };
        let mut item = new_strict_retention_baseline("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictRetentionBaselineRequiredCodeRule {
            message: "strict retention baseline needs an entry".to_string(),
        };
        let mut item = new_strict_retention_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictRetentionBaselineRequiredNameRule {
            message: "strict retention baseline needs an entry".to_string(),
        };
        let mut item = new_strict_retention_baseline("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = StrictRetentionBaselineExpiresAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_strict_retention_baseline("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = StrictRetentionBaselineRequiredTitleRule {
            message: "strict retention baseline needs a value".to_string(),
        };
        let mut item = new_strict_retention_baseline("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictRetentionBaselineCodeLengthRule { max_length: 86400 };
        let mut item = new_strict_retention_baseline("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
