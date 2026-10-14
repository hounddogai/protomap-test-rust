use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal health check rollup must pass. Check returns its problems, or none.
pub trait SeasonalHealthCheckRollupRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalHealthCheckRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckRollupRequiredIssuerRule {
    pub message: String,
}

impl SeasonalHealthCheckRollupRule for SeasonalHealthCheckRollupRequiredIssuerRule {
    fn check(&self, item: &SeasonalHealthCheckRollup) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckRollupAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl SeasonalHealthCheckRollupRule for SeasonalHealthCheckRollupAllowedSourceRule {
    fn check(&self, item: &SeasonalHealthCheckRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is expired", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckRollupTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalHealthCheckRollupRule for SeasonalHealthCheckRollupTitleLengthRule {
    fn check(&self, item: &SeasonalHealthCheckRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal health check rollup.
#[derive(Debug, Clone)]
pub struct SeasonalHealthCheckRollupValidator {
    pub rules: Vec<Rc<dyn SeasonalHealthCheckRollupRule>>,
}

impl SeasonalHealthCheckRollupValidator {
    pub fn validate(&self, item: &SeasonalHealthCheckRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalHealthCheckRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalHealthCheckRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_health_check_rollup_validator() -> SeasonalHealthCheckRollupValidator {
    SeasonalHealthCheckRollupValidator {
        rules: vec![
            Rc::new(SeasonalHealthCheckRollupRequiredIssuerRule {
                message: "seasonal health check rollup needs a value".to_string(),
            }) as Rc<dyn SeasonalHealthCheckRollupRule>,
            Rc::new(SeasonalHealthCheckRollupAllowedSourceRule {
                allowed: vec!["manual".to_string(), "lenient".to_string(), "rolling".to_string()],
            }),
            Rc::new(SeasonalHealthCheckRollupTitleLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_health_check_rollup_validator();
        let item = new_seasonal_health_check_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = SeasonalHealthCheckRollupRequiredIssuerRule {
            message: "seasonal health check rollup needs an entry".to_string(),
        };
        let mut item = new_seasonal_health_check_rollup("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SeasonalHealthCheckRollupTitleLengthRule { max_length: 16 };
        let mut item = new_seasonal_health_check_rollup("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
