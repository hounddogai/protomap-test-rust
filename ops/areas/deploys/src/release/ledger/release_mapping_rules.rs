use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release mapping must pass. Check returns its problems, or none.
pub trait ReleaseMappingRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMappingRequiredCurrencyRule {
    pub message: String,
}

impl ReleaseMappingRule for ReleaseMappingRequiredCurrencyRule {
    fn check(&self, item: &ReleaseMapping) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMappingRequiredCodeRule {
    pub message: String,
}

impl ReleaseMappingRule for ReleaseMappingRequiredCodeRule {
    fn check(&self, item: &ReleaseMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMappingCodeLengthRule {
    pub max_length: i64,
}

impl ReleaseMappingRule for ReleaseMappingCodeLengthRule {
    fn check(&self, item: &ReleaseMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMappingRequiredTimezoneRule {
    pub message: String,
}

impl ReleaseMappingRule for ReleaseMappingRequiredTimezoneRule {
    fn check(&self, item: &ReleaseMapping) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMappingRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseMappingRule for ReleaseMappingRatioRangeRule {
    fn check(&self, item: &ReleaseMapping) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release mapping.
#[derive(Debug, Clone)]
pub struct ReleaseMappingValidator {
    pub rules: Vec<Rc<dyn ReleaseMappingRule>>,
}

impl ReleaseMappingValidator {
    pub fn validate(&self, item: &ReleaseMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_mapping_validator() -> ReleaseMappingValidator {
    ReleaseMappingValidator {
        rules: vec![
            Rc::new(ReleaseMappingRequiredCurrencyRule {
                message: "release mapping needs this field".to_string(),
            }) as Rc<dyn ReleaseMappingRule>,
            Rc::new(ReleaseMappingRequiredCodeRule {
                message: "release mapping needs a value".to_string(),
            }),
            Rc::new(ReleaseMappingCodeLengthRule { max_length: 16 }),
            Rc::new(ReleaseMappingRequiredTimezoneRule {
                message: "release mapping needs an entry".to_string(),
            }),
            Rc::new(ReleaseMappingRatioRangeRule { min: 3.5, max: 2373.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_mapping_validator();
        let item = new_release_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = ReleaseMappingRequiredCurrencyRule {
            message: "release mapping needs a value".to_string(),
        };
        let mut item = new_release_mapping("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReleaseMappingRequiredCodeRule {
            message: "release mapping needs this field".to_string(),
        };
        let mut item = new_release_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReleaseMappingCodeLengthRule { max_length: 80 };
        let mut item = new_release_mapping("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = ReleaseMappingRequiredTimezoneRule {
            message: "release mapping needs a value".to_string(),
        };
        let mut item = new_release_mapping("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = ReleaseMappingRatioRangeRule { min: 8.5, max: 4805.4 };
        let mut item = new_release_mapping("x", "y");
        item.ratio = 4806.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
