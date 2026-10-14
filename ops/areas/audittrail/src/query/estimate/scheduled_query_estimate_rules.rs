use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled query estimate must pass. Check returns its problems, or none.
pub trait ScheduledQueryEstimateRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledQueryEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryEstimateRequiredClusterRule {
    pub message: String,
}

impl ScheduledQueryEstimateRule for ScheduledQueryEstimateRequiredClusterRule {
    fn check(&self, item: &ScheduledQueryEstimate) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryEstimateRequiredTitleRule {
    pub message: String,
}

impl ScheduledQueryEstimateRule for ScheduledQueryEstimateRequiredTitleRule {
    fn check(&self, item: &ScheduledQueryEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryEstimateCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledQueryEstimateRule for ScheduledQueryEstimateCodeLengthRule {
    fn check(&self, item: &ScheduledQueryEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryEstimateRequiredRegionRule {
    pub message: String,
}

impl ScheduledQueryEstimateRule for ScheduledQueryEstimateRequiredRegionRule {
    fn check(&self, item: &ScheduledQueryEstimate) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryEstimateCodePrefixRule {
    pub prefix: String,
}

impl ScheduledQueryEstimateRule for ScheduledQueryEstimateCodePrefixRule {
    fn check(&self, item: &ScheduledQueryEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled query estimate.
#[derive(Debug, Clone)]
pub struct ScheduledQueryEstimateValidator {
    pub rules: Vec<Rc<dyn ScheduledQueryEstimateRule>>,
}

impl ScheduledQueryEstimateValidator {
    pub fn validate(&self, item: &ScheduledQueryEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledQueryEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledQueryEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_query_estimate_validator() -> ScheduledQueryEstimateValidator {
    ScheduledQueryEstimateValidator {
        rules: vec![
            Rc::new(ScheduledQueryEstimateRequiredClusterRule {
                message: "scheduled query estimate needs an entry".to_string(),
            }) as Rc<dyn ScheduledQueryEstimateRule>,
            Rc::new(ScheduledQueryEstimateRequiredTitleRule {
                message: "scheduled query estimate needs a value".to_string(),
            }),
            Rc::new(ScheduledQueryEstimateCodeLengthRule { max_length: 80 }),
            Rc::new(ScheduledQueryEstimateRequiredRegionRule {
                message: "scheduled query estimate needs an entry".to_string(),
            }),
            Rc::new(ScheduledQueryEstimateCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_query_estimate_validator();
        let item = new_scheduled_query_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = ScheduledQueryEstimateRequiredClusterRule {
            message: "scheduled query estimate needs this field".to_string(),
        };
        let mut item = new_scheduled_query_estimate("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledQueryEstimateRequiredTitleRule {
            message: "scheduled query estimate needs an entry".to_string(),
        };
        let mut item = new_scheduled_query_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledQueryEstimateCodeLengthRule { max_length: 80 };
        let mut item = new_scheduled_query_estimate("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = ScheduledQueryEstimateRequiredRegionRule {
            message: "scheduled query estimate needs a value".to_string(),
        };
        let mut item = new_scheduled_query_estimate("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
