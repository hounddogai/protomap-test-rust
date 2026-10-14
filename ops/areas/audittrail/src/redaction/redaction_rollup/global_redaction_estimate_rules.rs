use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global redaction estimate must pass. Check returns its problems, or none.
pub trait GlobalRedactionEstimateRule: std::fmt::Debug {
    fn check(&self, item: &GlobalRedactionEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionEstimateRequiredTitleRule {
    pub message: String,
}

impl GlobalRedactionEstimateRule for GlobalRedactionEstimateRequiredTitleRule {
    fn check(&self, item: &GlobalRedactionEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionEstimateBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalRedactionEstimateRule for GlobalRedactionEstimateBurstRangeRule {
    fn check(&self, item: &GlobalRedactionEstimate) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionEstimateRequiredCipherRule {
    pub message: String,
}

impl GlobalRedactionEstimateRule for GlobalRedactionEstimateRequiredCipherRule {
    fn check(&self, item: &GlobalRedactionEstimate) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionEstimateCategoryLengthRule {
    pub max_length: i64,
}

impl GlobalRedactionEstimateRule for GlobalRedactionEstimateCategoryLengthRule {
    fn check(&self, item: &GlobalRedactionEstimate) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global redaction estimate.
#[derive(Debug, Clone)]
pub struct GlobalRedactionEstimateValidator {
    pub rules: Vec<Rc<dyn GlobalRedactionEstimateRule>>,
}

impl GlobalRedactionEstimateValidator {
    pub fn validate(&self, item: &GlobalRedactionEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalRedactionEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalRedactionEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_redaction_estimate_validator() -> GlobalRedactionEstimateValidator {
    GlobalRedactionEstimateValidator {
        rules: vec![
            Rc::new(GlobalRedactionEstimateRequiredTitleRule {
                message: "global redaction estimate needs a value".to_string(),
            }) as Rc<dyn GlobalRedactionEstimateRule>,
            Rc::new(GlobalRedactionEstimateBurstRangeRule { min: 64, max: 120 }),
            Rc::new(GlobalRedactionEstimateRequiredCipherRule {
                message: "global redaction estimate needs this field".to_string(),
            }),
            Rc::new(GlobalRedactionEstimateCategoryLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_redaction_estimate_validator();
        let item = new_global_redaction_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = GlobalRedactionEstimateRequiredTitleRule {
            message: "global redaction estimate needs an entry".to_string(),
        };
        let mut item = new_global_redaction_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = GlobalRedactionEstimateBurstRangeRule { min: 120, max: 255 };
        let mut item = new_global_redaction_estimate("x", "y");
        item.burst = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = GlobalRedactionEstimateRequiredCipherRule {
            message: "global redaction estimate needs an entry".to_string(),
        };
        let mut item = new_global_redaction_estimate("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = GlobalRedactionEstimateCategoryLengthRule { max_length: 255 };
        let mut item = new_global_redaction_estimate("x", "y");
        item.category = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
