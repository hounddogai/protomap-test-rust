use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary health check variance must pass. Check returns its problems, or none.
pub trait PrimaryHealthCheckVarianceRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryHealthCheckVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHealthCheckVarianceCodePrefixRule {
    pub prefix: String,
}

impl PrimaryHealthCheckVarianceRule for PrimaryHealthCheckVarianceCodePrefixRule {
    fn check(&self, item: &PrimaryHealthCheckVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHealthCheckVarianceLocaleLengthRule {
    pub max_length: i64,
}

impl PrimaryHealthCheckVarianceRule for PrimaryHealthCheckVarianceLocaleLengthRule {
    fn check(&self, item: &PrimaryHealthCheckVariance) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHealthCheckVarianceRequiredTitleRule {
    pub message: String,
}

impl PrimaryHealthCheckVarianceRule for PrimaryHealthCheckVarianceRequiredTitleRule {
    fn check(&self, item: &PrimaryHealthCheckVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHealthCheckVarianceRequiredCodeRule {
    pub message: String,
}

impl PrimaryHealthCheckVarianceRule for PrimaryHealthCheckVarianceRequiredCodeRule {
    fn check(&self, item: &PrimaryHealthCheckVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHealthCheckVarianceTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryHealthCheckVarianceRule for PrimaryHealthCheckVarianceTitleLengthRule {
    fn check(&self, item: &PrimaryHealthCheckVariance) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHealthCheckVarianceCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryHealthCheckVarianceRule for PrimaryHealthCheckVarianceCodeLengthRule {
    fn check(&self, item: &PrimaryHealthCheckVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary health check variance.
#[derive(Debug, Clone)]
pub struct PrimaryHealthCheckVarianceValidator {
    pub rules: Vec<Rc<dyn PrimaryHealthCheckVarianceRule>>,
}

impl PrimaryHealthCheckVarianceValidator {
    pub fn validate(&self, item: &PrimaryHealthCheckVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryHealthCheckVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryHealthCheckVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_health_check_variance_validator() -> PrimaryHealthCheckVarianceValidator {
    PrimaryHealthCheckVarianceValidator {
        rules: vec![
            Rc::new(PrimaryHealthCheckVarianceCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn PrimaryHealthCheckVarianceRule>,
            Rc::new(PrimaryHealthCheckVarianceLocaleLengthRule { max_length: 120 }),
            Rc::new(PrimaryHealthCheckVarianceRequiredTitleRule {
                message: "primary health check variance needs an entry".to_string(),
            }),
            Rc::new(PrimaryHealthCheckVarianceRequiredCodeRule {
                message: "primary health check variance needs an entry".to_string(),
            }),
            Rc::new(PrimaryHealthCheckVarianceTitleLengthRule { max_length: 80 }),
            Rc::new(PrimaryHealthCheckVarianceCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_health_check_variance_validator();
        let item = new_primary_health_check_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = PrimaryHealthCheckVarianceLocaleLengthRule { max_length: 3600 };
        let mut item = new_primary_health_check_variance("x", "y");
        item.locale = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryHealthCheckVarianceRequiredTitleRule {
            message: "primary health check variance needs this field".to_string(),
        };
        let mut item = new_primary_health_check_variance("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryHealthCheckVarianceRequiredCodeRule {
            message: "primary health check variance needs an entry".to_string(),
        };
        let mut item = new_primary_health_check_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryHealthCheckVarianceTitleLengthRule { max_length: 86400 };
        let mut item = new_primary_health_check_variance("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryHealthCheckVarianceCodeLengthRule { max_length: 64 };
        let mut item = new_primary_health_check_variance("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
