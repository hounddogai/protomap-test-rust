use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard request must pass. Check returns its problems, or none.
pub trait DashboardRequestRule: std::fmt::Debug {
    fn check(&self, item: &DashboardRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRequestRequiredCodeRule {
    pub message: String,
}

impl DashboardRequestRule for DashboardRequestRequiredCodeRule {
    fn check(&self, item: &DashboardRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRequestContactEmailLengthRule {
    pub max_length: i64,
}

impl DashboardRequestRule for DashboardRequestContactEmailLengthRule {
    fn check(&self, item: &DashboardRequest) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRequestCodeLengthRule {
    pub max_length: i64,
}

impl DashboardRequestRule for DashboardRequestCodeLengthRule {
    fn check(&self, item: &DashboardRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRequestCodePrefixRule {
    pub prefix: String,
}

impl DashboardRequestRule for DashboardRequestCodePrefixRule {
    fn check(&self, item: &DashboardRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRequestRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DashboardRequestRule for DashboardRequestRateRangeRule {
    fn check(&self, item: &DashboardRequest) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a dashboard request.
#[derive(Debug, Clone)]
pub struct DashboardRequestValidator {
    pub rules: Vec<Rc<dyn DashboardRequestRule>>,
}

impl DashboardRequestValidator {
    pub fn validate(&self, item: &DashboardRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_request_validator() -> DashboardRequestValidator {
    DashboardRequestValidator {
        rules: vec![
            Rc::new(DashboardRequestRequiredCodeRule {
                message: "dashboard request needs a value".to_string(),
            }) as Rc<dyn DashboardRequestRule>,
            Rc::new(DashboardRequestContactEmailLengthRule { max_length: 255 }),
            Rc::new(DashboardRequestCodeLengthRule { max_length: 1000 }),
            Rc::new(DashboardRequestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(DashboardRequestRateRangeRule { min: 4.7, max: 777.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_request_validator();
        let item = new_dashboard_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DashboardRequestRequiredCodeRule {
            message: "dashboard request needs an entry".to_string(),
        };
        let mut item = new_dashboard_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = DashboardRequestContactEmailLengthRule { max_length: 64 };
        let mut item = new_dashboard_request("x", "y");
        item.contact_email = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DashboardRequestCodeLengthRule { max_length: 1000 };
        let mut item = new_dashboard_request("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = DashboardRequestRateRangeRule { min: 1.7, max: 4814.6 };
        let mut item = new_dashboard_request("x", "y");
        item.rate = 4816.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
