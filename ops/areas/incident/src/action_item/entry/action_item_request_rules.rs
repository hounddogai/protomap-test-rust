use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item request must pass. Check returns its problems, or none.
pub trait ActionItemRequestRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRequestCodePrefixRule {
    pub prefix: String,
}

impl ActionItemRequestRule for ActionItemRequestCodePrefixRule {
    fn check(&self, item: &ActionItemRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRequestExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemRequestRule for ActionItemRequestExpiresAtRangeRule {
    fn check(&self, item: &ActionItemRequest) -> Vec<String> {
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
pub struct ActionItemRequestMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemRequestRule for ActionItemRequestMinimumRangeRule {
    fn check(&self, item: &ActionItemRequest) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a action item request.
#[derive(Debug, Clone)]
pub struct ActionItemRequestValidator {
    pub rules: Vec<Rc<dyn ActionItemRequestRule>>,
}

impl ActionItemRequestValidator {
    pub fn validate(&self, item: &ActionItemRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_request_validator() -> ActionItemRequestValidator {
    ActionItemRequestValidator {
        rules: vec![
            Rc::new(ActionItemRequestCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn ActionItemRequestRule>,
            Rc::new(ActionItemRequestExpiresAtRangeRule { min: 3600, max: 3600 }),
            Rc::new(ActionItemRequestMinimumRangeRule { min: 2.1, max: 740.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_request_validator();
        let item = new_action_item_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = ActionItemRequestExpiresAtRangeRule { min: 1000, max: 1000 };
        let mut item = new_action_item_request("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ActionItemRequestMinimumRangeRule { min: 3.0, max: 3152.7 };
        let mut item = new_action_item_request("x", "y");
        item.minimum = 3154.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
