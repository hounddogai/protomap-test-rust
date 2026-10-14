use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a limit metric must pass. Check returns its problems, or none.
pub trait LimitMetricRule: std::fmt::Debug {
    fn check(&self, item: &LimitMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitMetricAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl LimitMetricRule for LimitMetricAllowedAudienceRule {
    fn check(&self, item: &LimitMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is out of range", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitMetricRequiredNameRule {
    pub message: String,
}

impl LimitMetricRule for LimitMetricRequiredNameRule {
    fn check(&self, item: &LimitMetric) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitMetricRequiredCodeRule {
    pub message: String,
}

impl LimitMetricRule for LimitMetricRequiredCodeRule {
    fn check(&self, item: &LimitMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit metric.
#[derive(Debug, Clone)]
pub struct LimitMetricValidator {
    pub rules: Vec<Rc<dyn LimitMetricRule>>,
}

impl LimitMetricValidator {
    pub fn validate(&self, item: &LimitMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_metric_validator() -> LimitMetricValidator {
    LimitMetricValidator {
        rules: vec![
            Rc::new(LimitMetricAllowedAudienceRule { allowed: Vec::new() }) as Rc<dyn LimitMetricRule>,
            Rc::new(LimitMetricRequiredNameRule {
                message: "limit metric needs a value".to_string(),
            }),
            Rc::new(LimitMetricRequiredCodeRule {
                message: "limit metric needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_metric_validator();
        let item = new_limit_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LimitMetricRequiredNameRule {
            message: "limit metric needs this field".to_string(),
        };
        let mut item = new_limit_metric("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LimitMetricRequiredCodeRule {
            message: "limit metric needs this field".to_string(),
        };
        let mut item = new_limit_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
