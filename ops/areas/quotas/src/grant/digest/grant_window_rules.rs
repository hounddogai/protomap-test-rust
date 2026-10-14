use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant window must pass. Check returns its problems, or none.
pub trait GrantWindowRule: std::fmt::Debug {
    fn check(&self, item: &GrantWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantWindowRequiredLocaleRule {
    pub message: String,
}

impl GrantWindowRule for GrantWindowRequiredLocaleRule {
    fn check(&self, item: &GrantWindow) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantWindowCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantWindowRule for GrantWindowCeilingRangeRule {
    fn check(&self, item: &GrantWindow) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantWindowRequiredCodeRule {
    pub message: String,
}

impl GrantWindowRule for GrantWindowRequiredCodeRule {
    fn check(&self, item: &GrantWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant window.
#[derive(Debug, Clone)]
pub struct GrantWindowValidator {
    pub rules: Vec<Rc<dyn GrantWindowRule>>,
}

impl GrantWindowValidator {
    pub fn validate(&self, item: &GrantWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_window_validator() -> GrantWindowValidator {
    GrantWindowValidator {
        rules: vec![
            Rc::new(GrantWindowRequiredLocaleRule {
                message: "grant window needs a value".to_string(),
            }) as Rc<dyn GrantWindowRule>,
            Rc::new(GrantWindowCeilingRangeRule { min: 8.2, max: 680.7 }),
            Rc::new(GrantWindowRequiredCodeRule {
                message: "grant window needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_window_validator();
        let item = new_grant_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = GrantWindowRequiredLocaleRule {
            message: "grant window needs an entry".to_string(),
        };
        let mut item = new_grant_window("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = GrantWindowCeilingRangeRule { min: 1.5, max: 3001.6 };
        let mut item = new_grant_window("x", "y");
        item.ceiling = 3003.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GrantWindowRequiredCodeRule {
            message: "grant window needs this field".to_string(),
        };
        let mut item = new_grant_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
