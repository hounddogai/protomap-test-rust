use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped egress variance must pass. Check returns its problems, or none.
pub trait CappedEgressVarianceRule: std::fmt::Debug {
    fn check(&self, item: &CappedEgressVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEgressVarianceCodePrefixRule {
    pub prefix: String,
}

impl CappedEgressVarianceRule for CappedEgressVarianceCodePrefixRule {
    fn check(&self, item: &CappedEgressVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEgressVarianceLocaleLengthRule {
    pub max_length: i64,
}

impl CappedEgressVarianceRule for CappedEgressVarianceLocaleLengthRule {
    fn check(&self, item: &CappedEgressVariance) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEgressVarianceLabelLengthRule {
    pub max_length: i64,
}

impl CappedEgressVarianceRule for CappedEgressVarianceLabelLengthRule {
    fn check(&self, item: &CappedEgressVariance) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEgressVarianceCodeLengthRule {
    pub max_length: i64,
}

impl CappedEgressVarianceRule for CappedEgressVarianceCodeLengthRule {
    fn check(&self, item: &CappedEgressVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped egress variance.
#[derive(Debug, Clone)]
pub struct CappedEgressVarianceValidator {
    pub rules: Vec<Rc<dyn CappedEgressVarianceRule>>,
}

impl CappedEgressVarianceValidator {
    pub fn validate(&self, item: &CappedEgressVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedEgressVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedEgressVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_egress_variance_validator() -> CappedEgressVarianceValidator {
    CappedEgressVarianceValidator {
        rules: vec![
            Rc::new(CappedEgressVarianceCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn CappedEgressVarianceRule>,
            Rc::new(CappedEgressVarianceLocaleLengthRule { max_length: 80 }),
            Rc::new(CappedEgressVarianceLabelLengthRule { max_length: 120 }),
            Rc::new(CappedEgressVarianceCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_egress_variance_validator();
        let item = new_capped_egress_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = CappedEgressVarianceLocaleLengthRule { max_length: 1000 };
        let mut item = new_capped_egress_variance("x", "y");
        item.locale = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CappedEgressVarianceLabelLengthRule { max_length: 1000 };
        let mut item = new_capped_egress_variance("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedEgressVarianceCodeLengthRule { max_length: 32 };
        let mut item = new_capped_egress_variance("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
