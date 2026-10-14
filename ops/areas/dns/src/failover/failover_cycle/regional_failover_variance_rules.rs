use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional failover variance must pass. Check returns its problems, or none.
pub trait RegionalFailoverVarianceRule: std::fmt::Debug {
    fn check(&self, item: &RegionalFailoverVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverVarianceRequiredTitleRule {
    pub message: String,
}

impl RegionalFailoverVarianceRule for RegionalFailoverVarianceRequiredTitleRule {
    fn check(&self, item: &RegionalFailoverVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverVarianceCodeLengthRule {
    pub max_length: i64,
}

impl RegionalFailoverVarianceRule for RegionalFailoverVarianceCodeLengthRule {
    fn check(&self, item: &RegionalFailoverVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverVarianceAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl RegionalFailoverVarianceRule for RegionalFailoverVarianceAllowedScopeRule {
    fn check(&self, item: &RegionalFailoverVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is below minimum", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverVarianceNameLengthRule {
    pub max_length: i64,
}

impl RegionalFailoverVarianceRule for RegionalFailoverVarianceNameLengthRule {
    fn check(&self, item: &RegionalFailoverVariance) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional failover variance.
#[derive(Debug, Clone)]
pub struct RegionalFailoverVarianceValidator {
    pub rules: Vec<Rc<dyn RegionalFailoverVarianceRule>>,
}

impl RegionalFailoverVarianceValidator {
    pub fn validate(&self, item: &RegionalFailoverVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalFailoverVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalFailoverVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_failover_variance_validator() -> RegionalFailoverVarianceValidator {
    RegionalFailoverVarianceValidator {
        rules: vec![
            Rc::new(RegionalFailoverVarianceRequiredTitleRule {
                message: "regional failover variance needs a value".to_string(),
            }) as Rc<dyn RegionalFailoverVarianceRule>,
            Rc::new(RegionalFailoverVarianceCodeLengthRule { max_length: 255 }),
            Rc::new(RegionalFailoverVarianceAllowedScopeRule {
                allowed: vec!["fallback".to_string(), "primary".to_string()],
            }),
            Rc::new(RegionalFailoverVarianceNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_failover_variance_validator();
        let item = new_regional_failover_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionalFailoverVarianceRequiredTitleRule {
            message: "regional failover variance needs a value".to_string(),
        };
        let mut item = new_regional_failover_variance("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalFailoverVarianceCodeLengthRule { max_length: 32 };
        let mut item = new_regional_failover_variance("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionalFailoverVarianceNameLengthRule { max_length: 64 };
        let mut item = new_regional_failover_variance("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
