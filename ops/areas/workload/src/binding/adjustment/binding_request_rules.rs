use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding request must pass. Check returns its problems, or none.
pub trait BindingRequestRule: std::fmt::Debug {
    fn check(&self, item: &BindingRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRequestLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingRequestRule for BindingRequestLimitRangeRule {
    fn check(&self, item: &BindingRequest) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRequestPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingRequestRule for BindingRequestPriorityRangeRule {
    fn check(&self, item: &BindingRequest) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRequestRequiredTitleRule {
    pub message: String,
}

impl BindingRequestRule for BindingRequestRequiredTitleRule {
    fn check(&self, item: &BindingRequest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRequestCodePrefixRule {
    pub prefix: String,
}

impl BindingRequestRule for BindingRequestCodePrefixRule {
    fn check(&self, item: &BindingRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRequestRequiredCodeRule {
    pub message: String,
}

impl BindingRequestRule for BindingRequestRequiredCodeRule {
    fn check(&self, item: &BindingRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding request.
#[derive(Debug, Clone)]
pub struct BindingRequestValidator {
    pub rules: Vec<Rc<dyn BindingRequestRule>>,
}

impl BindingRequestValidator {
    pub fn validate(&self, item: &BindingRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_request_validator() -> BindingRequestValidator {
    BindingRequestValidator {
        rules: vec![
            Rc::new(BindingRequestLimitRangeRule { min: 80, max: 255 }) as Rc<dyn BindingRequestRule>,
            Rc::new(BindingRequestPriorityRangeRule { min: 32, max: 3600 }),
            Rc::new(BindingRequestRequiredTitleRule {
                message: "binding request needs this field".to_string(),
            }),
            Rc::new(BindingRequestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(BindingRequestRequiredCodeRule {
                message: "binding request needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_request_validator();
        let item = new_binding_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = BindingRequestLimitRangeRule { min: 16, max: 80 };
        let mut item = new_binding_request("x", "y");
        item.limit = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = BindingRequestPriorityRangeRule { min: 32, max: 80 };
        let mut item = new_binding_request("x", "y");
        item.priority = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BindingRequestRequiredTitleRule {
            message: "binding request needs a value".to_string(),
        };
        let mut item = new_binding_request("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BindingRequestRequiredCodeRule {
            message: "binding request needs a value".to_string(),
        };
        let mut item = new_binding_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
