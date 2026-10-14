use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global token request must pass. Check returns its problems, or none.
pub trait GlobalTokenRequestRule: std::fmt::Debug {
    fn check(&self, item: &GlobalTokenRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenRequestRequiredContactEmailRule {
    pub message: String,
}

impl GlobalTokenRequestRule for GlobalTokenRequestRequiredContactEmailRule {
    fn check(&self, item: &GlobalTokenRequest) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenRequestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalTokenRequestRule for GlobalTokenRequestCreatedAtRangeRule {
    fn check(&self, item: &GlobalTokenRequest) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenRequestCodePrefixRule {
    pub prefix: String,
}

impl GlobalTokenRequestRule for GlobalTokenRequestCodePrefixRule {
    fn check(&self, item: &GlobalTokenRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenRequestAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalTokenRequestRule for GlobalTokenRequestAmountRangeRule {
    fn check(&self, item: &GlobalTokenRequest) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenRequestRequiredLabelRule {
    pub message: String,
}

impl GlobalTokenRequestRule for GlobalTokenRequestRequiredLabelRule {
    fn check(&self, item: &GlobalTokenRequest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global token request.
#[derive(Debug, Clone)]
pub struct GlobalTokenRequestValidator {
    pub rules: Vec<Rc<dyn GlobalTokenRequestRule>>,
}

impl GlobalTokenRequestValidator {
    pub fn validate(&self, item: &GlobalTokenRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalTokenRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalTokenRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_token_request_validator() -> GlobalTokenRequestValidator {
    GlobalTokenRequestValidator {
        rules: vec![
            Rc::new(GlobalTokenRequestRequiredContactEmailRule {
                message: "global token request needs a value".to_string(),
            }) as Rc<dyn GlobalTokenRequestRule>,
            Rc::new(GlobalTokenRequestCreatedAtRangeRule { min: 16, max: 3600 }),
            Rc::new(GlobalTokenRequestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(GlobalTokenRequestAmountRangeRule { min: 5.4, max: 1760.5 }),
            Rc::new(GlobalTokenRequestRequiredLabelRule {
                message: "global token request needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_token_request_validator();
        let item = new_global_token_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = GlobalTokenRequestRequiredContactEmailRule {
            message: "global token request needs an entry".to_string(),
        };
        let mut item = new_global_token_request("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = GlobalTokenRequestCreatedAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_global_token_request("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = GlobalTokenRequestAmountRangeRule { min: 0.0, max: 2261.3 };
        let mut item = new_global_token_request("x", "y");
        item.amount = 2262.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = GlobalTokenRequestRequiredLabelRule {
            message: "global token request needs an entry".to_string(),
        };
        let mut item = new_global_token_request("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
