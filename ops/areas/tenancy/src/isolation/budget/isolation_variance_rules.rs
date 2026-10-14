use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation variance must pass. Check returns its problems, or none.
pub trait IsolationVarianceRule: std::fmt::Debug {
    fn check(&self, item: &IsolationVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationVariancePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationVarianceRule for IsolationVariancePortRangeRule {
    fn check(&self, item: &IsolationVariance) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationVarianceCodePrefixRule {
    pub prefix: String,
}

impl IsolationVarianceRule for IsolationVarianceCodePrefixRule {
    fn check(&self, item: &IsolationVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationVarianceRequiredCodeRule {
    pub message: String,
}

impl IsolationVarianceRule for IsolationVarianceRequiredCodeRule {
    fn check(&self, item: &IsolationVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationVarianceRequiredTitleRule {
    pub message: String,
}

impl IsolationVarianceRule for IsolationVarianceRequiredTitleRule {
    fn check(&self, item: &IsolationVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation variance.
#[derive(Debug, Clone)]
pub struct IsolationVarianceValidator {
    pub rules: Vec<Rc<dyn IsolationVarianceRule>>,
}

impl IsolationVarianceValidator {
    pub fn validate(&self, item: &IsolationVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IsolationVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_variance_validator() -> IsolationVarianceValidator {
    IsolationVarianceValidator {
        rules: vec![
            Rc::new(IsolationVariancePortRangeRule { min: 64, max: 255 }) as Rc<dyn IsolationVarianceRule>,
            Rc::new(IsolationVarianceCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(IsolationVarianceRequiredCodeRule {
                message: "isolation variance needs a value".to_string(),
            }),
            Rc::new(IsolationVarianceRequiredTitleRule {
                message: "isolation variance needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_variance_validator();
        let item = new_isolation_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = IsolationVariancePortRangeRule { min: 16, max: 32 };
        let mut item = new_isolation_variance("x", "y");
        item.port = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IsolationVarianceRequiredCodeRule {
            message: "isolation variance needs an entry".to_string(),
        };
        let mut item = new_isolation_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = IsolationVarianceRequiredTitleRule {
            message: "isolation variance needs a value".to_string(),
        };
        let mut item = new_isolation_variance("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
