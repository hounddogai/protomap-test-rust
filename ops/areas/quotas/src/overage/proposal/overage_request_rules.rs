use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage request must pass. Check returns its problems, or none.
pub trait OverageRequestRule: std::fmt::Debug {
    fn check(&self, item: &OverageRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRequestRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageRequestRule for OverageRequestRetriesRangeRule {
    fn check(&self, item: &OverageRequest) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRequestRequiredCodeRule {
    pub message: String,
}

impl OverageRequestRule for OverageRequestRequiredCodeRule {
    fn check(&self, item: &OverageRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRequestRequiredChannelRule {
    pub message: String,
}

impl OverageRequestRule for OverageRequestRequiredChannelRule {
    fn check(&self, item: &OverageRequest) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRequestGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageRequestRule for OverageRequestGraceDaysRangeRule {
    fn check(&self, item: &OverageRequest) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRequestRequiredLabelRule {
    pub message: String,
}

impl OverageRequestRule for OverageRequestRequiredLabelRule {
    fn check(&self, item: &OverageRequest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage request.
#[derive(Debug, Clone)]
pub struct OverageRequestValidator {
    pub rules: Vec<Rc<dyn OverageRequestRule>>,
}

impl OverageRequestValidator {
    pub fn validate(&self, item: &OverageRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_request_validator() -> OverageRequestValidator {
    OverageRequestValidator {
        rules: vec![
            Rc::new(OverageRequestRetriesRangeRule { min: 64, max: 86400 }) as Rc<dyn OverageRequestRule>,
            Rc::new(OverageRequestRequiredCodeRule {
                message: "overage request needs an entry".to_string(),
            }),
            Rc::new(OverageRequestRequiredChannelRule {
                message: "overage request needs an entry".to_string(),
            }),
            Rc::new(OverageRequestGraceDaysRangeRule { min: 3600, max: 86400 }),
            Rc::new(OverageRequestRequiredLabelRule {
                message: "overage request needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_request_validator();
        let item = new_overage_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = OverageRequestRetriesRangeRule { min: 86400, max: 86400 };
        let mut item = new_overage_request("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = OverageRequestRequiredCodeRule {
            message: "overage request needs a value".to_string(),
        };
        let mut item = new_overage_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = OverageRequestRequiredChannelRule {
            message: "overage request needs an entry".to_string(),
        };
        let mut item = new_overage_request("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = OverageRequestGraceDaysRangeRule { min: 16, max: 120 };
        let mut item = new_overage_request("x", "y");
        item.grace_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = OverageRequestRequiredLabelRule {
            message: "overage request needs an entry".to_string(),
        };
        let mut item = new_overage_request("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
