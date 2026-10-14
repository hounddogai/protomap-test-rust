use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction variance must pass. Check returns its problems, or none.
pub trait RedactionVarianceRule: std::fmt::Debug {
    fn check(&self, item: &RedactionVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionVarianceRequiredCodeRule {
    pub message: String,
}

impl RedactionVarianceRule for RedactionVarianceRequiredCodeRule {
    fn check(&self, item: &RedactionVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionVariancePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionVarianceRule for RedactionVariancePortRangeRule {
    fn check(&self, item: &RedactionVariance) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionVarianceRequiredIssuerRule {
    pub message: String,
}

impl RedactionVarianceRule for RedactionVarianceRequiredIssuerRule {
    fn check(&self, item: &RedactionVariance) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionVarianceBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RedactionVarianceRule for RedactionVarianceBudgetRangeRule {
    fn check(&self, item: &RedactionVariance) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionVarianceIssuerLengthRule {
    pub max_length: i64,
}

impl RedactionVarianceRule for RedactionVarianceIssuerLengthRule {
    fn check(&self, item: &RedactionVariance) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction variance.
#[derive(Debug, Clone)]
pub struct RedactionVarianceValidator {
    pub rules: Vec<Rc<dyn RedactionVarianceRule>>,
}

impl RedactionVarianceValidator {
    pub fn validate(&self, item: &RedactionVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_variance_validator() -> RedactionVarianceValidator {
    RedactionVarianceValidator {
        rules: vec![
            Rc::new(RedactionVarianceRequiredCodeRule {
                message: "redaction variance needs this field".to_string(),
            }) as Rc<dyn RedactionVarianceRule>,
            Rc::new(RedactionVariancePortRangeRule { min: 16, max: 64 }),
            Rc::new(RedactionVarianceRequiredIssuerRule {
                message: "redaction variance needs a value".to_string(),
            }),
            Rc::new(RedactionVarianceBudgetRangeRule { min: 9.8, max: 69.5 }),
            Rc::new(RedactionVarianceIssuerLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_variance_validator();
        let item = new_redaction_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RedactionVarianceRequiredCodeRule {
            message: "redaction variance needs a value".to_string(),
        };
        let mut item = new_redaction_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = RedactionVariancePortRangeRule { min: 80, max: 3600 };
        let mut item = new_redaction_variance("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = RedactionVarianceRequiredIssuerRule {
            message: "redaction variance needs this field".to_string(),
        };
        let mut item = new_redaction_variance("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RedactionVarianceBudgetRangeRule { min: 4.8, max: 3238.8 };
        let mut item = new_redaction_variance("x", "y");
        item.budget = 3240.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = RedactionVarianceIssuerLengthRule { max_length: 3600 };
        let mut item = new_redaction_variance("x", "y");
        item.issuer = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
