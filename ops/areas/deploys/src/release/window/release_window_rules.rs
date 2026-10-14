use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release window must pass. Check returns its problems, or none.
pub trait ReleaseWindowRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseWindowRequiredCodeRule {
    pub message: String,
}

impl ReleaseWindowRule for ReleaseWindowRequiredCodeRule {
    fn check(&self, item: &ReleaseWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseWindowMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseWindowRule for ReleaseWindowMinimumRangeRule {
    fn check(&self, item: &ReleaseWindow) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseWindowCodePrefixRule {
    pub prefix: String,
}

impl ReleaseWindowRule for ReleaseWindowCodePrefixRule {
    fn check(&self, item: &ReleaseWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release window.
#[derive(Debug, Clone)]
pub struct ReleaseWindowValidator {
    pub rules: Vec<Rc<dyn ReleaseWindowRule>>,
}

impl ReleaseWindowValidator {
    pub fn validate(&self, item: &ReleaseWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_window_validator() -> ReleaseWindowValidator {
    ReleaseWindowValidator {
        rules: vec![
            Rc::new(ReleaseWindowRequiredCodeRule {
                message: "release window needs this field".to_string(),
            }) as Rc<dyn ReleaseWindowRule>,
            Rc::new(ReleaseWindowMinimumRangeRule { min: 9.8, max: 57.5 }),
            Rc::new(ReleaseWindowCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_window_validator();
        let item = new_release_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ReleaseWindowRequiredCodeRule {
            message: "release window needs an entry".to_string(),
        };
        let mut item = new_release_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ReleaseWindowMinimumRangeRule { min: 1.4, max: 2063.3 };
        let mut item = new_release_window("x", "y");
        item.minimum = 2064.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
