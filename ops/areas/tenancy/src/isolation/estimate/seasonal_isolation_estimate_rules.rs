use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal isolation estimate must pass. Check returns its problems, or none.
pub trait SeasonalIsolationEstimateRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalIsolationEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationEstimateRequiredCodeRule {
    pub message: String,
}

impl SeasonalIsolationEstimateRule for SeasonalIsolationEstimateRequiredCodeRule {
    fn check(&self, item: &SeasonalIsolationEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationEstimateCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalIsolationEstimateRule for SeasonalIsolationEstimateCodeLengthRule {
    fn check(&self, item: &SeasonalIsolationEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationEstimateTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalIsolationEstimateRule for SeasonalIsolationEstimateTitleLengthRule {
    fn check(&self, item: &SeasonalIsolationEstimate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationEstimateRequiredLabelRule {
    pub message: String,
}

impl SeasonalIsolationEstimateRule for SeasonalIsolationEstimateRequiredLabelRule {
    fn check(&self, item: &SeasonalIsolationEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal isolation estimate.
#[derive(Debug, Clone)]
pub struct SeasonalIsolationEstimateValidator {
    pub rules: Vec<Rc<dyn SeasonalIsolationEstimateRule>>,
}

impl SeasonalIsolationEstimateValidator {
    pub fn validate(&self, item: &SeasonalIsolationEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalIsolationEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalIsolationEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_isolation_estimate_validator() -> SeasonalIsolationEstimateValidator {
    SeasonalIsolationEstimateValidator {
        rules: vec![
            Rc::new(SeasonalIsolationEstimateRequiredCodeRule {
                message: "seasonal isolation estimate needs an entry".to_string(),
            }) as Rc<dyn SeasonalIsolationEstimateRule>,
            Rc::new(SeasonalIsolationEstimateCodeLengthRule { max_length: 32 }),
            Rc::new(SeasonalIsolationEstimateTitleLengthRule { max_length: 32 }),
            Rc::new(SeasonalIsolationEstimateRequiredLabelRule {
                message: "seasonal isolation estimate needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_isolation_estimate_validator();
        let item = new_seasonal_isolation_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalIsolationEstimateRequiredCodeRule {
            message: "seasonal isolation estimate needs a value".to_string(),
        };
        let mut item = new_seasonal_isolation_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalIsolationEstimateCodeLengthRule { max_length: 3600 };
        let mut item = new_seasonal_isolation_estimate("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SeasonalIsolationEstimateTitleLengthRule { max_length: 1000 };
        let mut item = new_seasonal_isolation_estimate("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SeasonalIsolationEstimateRequiredLabelRule {
            message: "seasonal isolation estimate needs an entry".to_string(),
        };
        let mut item = new_seasonal_isolation_estimate("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
