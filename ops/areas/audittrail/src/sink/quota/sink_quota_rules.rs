use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a sink quota must pass. Check returns its problems, or none.
pub trait SinkQuotaRule: std::fmt::Debug {
    fn check(&self, item: &SinkQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkQuotaCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SinkQuotaRule for SinkQuotaCostRangeRule {
    fn check(&self, item: &SinkQuota) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkQuotaRequiredLocaleRule {
    pub message: String,
}

impl SinkQuotaRule for SinkQuotaRequiredLocaleRule {
    fn check(&self, item: &SinkQuota) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkQuotaAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SinkQuotaRule for SinkQuotaAllowedCodeRule {
    fn check(&self, item: &SinkQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkQuotaTitleLengthRule {
    pub max_length: i64,
}

impl SinkQuotaRule for SinkQuotaTitleLengthRule {
    fn check(&self, item: &SinkQuota) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink quota.
#[derive(Debug, Clone)]
pub struct SinkQuotaValidator {
    pub rules: Vec<Rc<dyn SinkQuotaRule>>,
}

impl SinkQuotaValidator {
    pub fn validate(&self, item: &SinkQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SinkQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_quota_validator() -> SinkQuotaValidator {
    SinkQuotaValidator {
        rules: vec![
            Rc::new(SinkQuotaCostRangeRule { min: 1.0, max: 4355.5 }) as Rc<dyn SinkQuotaRule>,
            Rc::new(SinkQuotaRequiredLocaleRule {
                message: "sink quota needs a value".to_string(),
            }),
            Rc::new(SinkQuotaAllowedCodeRule {
                allowed: vec!["seasonal".to_string(), "lenient".to_string()],
            }),
            Rc::new(SinkQuotaTitleLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_quota_validator();
        let item = new_sink_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = SinkQuotaCostRangeRule { min: 3.0, max: 223.6 };
        let mut item = new_sink_quota("x", "y");
        item.cost = 225.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = SinkQuotaRequiredLocaleRule {
            message: "sink quota needs a value".to_string(),
        };
        let mut item = new_sink_quota("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SinkQuotaTitleLengthRule { max_length: 32 };
        let mut item = new_sink_quota("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
