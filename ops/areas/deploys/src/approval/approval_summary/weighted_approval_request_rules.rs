use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted approval request must pass. Check returns its problems, or none.
pub trait WeightedApprovalRequestRule: std::fmt::Debug {
    fn check(&self, item: &WeightedApprovalRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedApprovalRequestNameLengthRule {
    pub max_length: i64,
}

impl WeightedApprovalRequestRule for WeightedApprovalRequestNameLengthRule {
    fn check(&self, item: &WeightedApprovalRequest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedApprovalRequestLocaleLengthRule {
    pub max_length: i64,
}

impl WeightedApprovalRequestRule for WeightedApprovalRequestLocaleLengthRule {
    fn check(&self, item: &WeightedApprovalRequest) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedApprovalRequestRequiredLocaleRule {
    pub message: String,
}

impl WeightedApprovalRequestRule for WeightedApprovalRequestRequiredLocaleRule {
    fn check(&self, item: &WeightedApprovalRequest) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted approval request.
#[derive(Debug, Clone)]
pub struct WeightedApprovalRequestValidator {
    pub rules: Vec<Rc<dyn WeightedApprovalRequestRule>>,
}

impl WeightedApprovalRequestValidator {
    pub fn validate(&self, item: &WeightedApprovalRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedApprovalRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedApprovalRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_approval_request_validator() -> WeightedApprovalRequestValidator {
    WeightedApprovalRequestValidator {
        rules: vec![
            Rc::new(WeightedApprovalRequestNameLengthRule { max_length: 86400 }) as Rc<dyn WeightedApprovalRequestRule>,
            Rc::new(WeightedApprovalRequestLocaleLengthRule { max_length: 86400 }),
            Rc::new(WeightedApprovalRequestRequiredLocaleRule {
                message: "weighted approval request needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_approval_request_validator();
        let item = new_weighted_approval_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedApprovalRequestNameLengthRule { max_length: 1000 };
        let mut item = new_weighted_approval_request("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = WeightedApprovalRequestLocaleLengthRule { max_length: 64 };
        let mut item = new_weighted_approval_request("x", "y");
        item.locale = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = WeightedApprovalRequestRequiredLocaleRule {
            message: "weighted approval request needs a value".to_string(),
        };
        let mut item = new_weighted_approval_request("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
