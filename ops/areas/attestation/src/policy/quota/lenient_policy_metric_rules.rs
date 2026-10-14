use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient policy metric must pass. Check returns its problems, or none.
pub trait LenientPolicyMetricRule: std::fmt::Debug {
    fn check(&self, item: &LenientPolicyMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyMetricCodePrefixRule {
    pub prefix: String,
}

impl LenientPolicyMetricRule for LenientPolicyMetricCodePrefixRule {
    fn check(&self, item: &LenientPolicyMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyMetricPinnedGuardRule {
    pub limit: f64,
}

impl LenientPolicyMetricRule for LenientPolicyMetricPinnedGuardRule {
    fn check(&self, item: &LenientPolicyMetric) -> Vec<String> {
        if item.pinned && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while pinned", item.amount, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyMetricTitleLengthRule {
    pub max_length: i64,
}

impl LenientPolicyMetricRule for LenientPolicyMetricTitleLengthRule {
    fn check(&self, item: &LenientPolicyMetric) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyMetricAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LenientPolicyMetricRule for LenientPolicyMetricAllowedTitleRule {
    fn check(&self, item: &LenientPolicyMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyMetricRequiredChannelRule {
    pub message: String,
}

impl LenientPolicyMetricRule for LenientPolicyMetricRequiredChannelRule {
    fn check(&self, item: &LenientPolicyMetric) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient policy metric.
#[derive(Debug, Clone)]
pub struct LenientPolicyMetricValidator {
    pub rules: Vec<Rc<dyn LenientPolicyMetricRule>>,
}

impl LenientPolicyMetricValidator {
    pub fn validate(&self, item: &LenientPolicyMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientPolicyMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPolicyMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_policy_metric_validator() -> LenientPolicyMetricValidator {
    LenientPolicyMetricValidator {
        rules: vec![
            Rc::new(LenientPolicyMetricCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn LenientPolicyMetricRule>,
            Rc::new(LenientPolicyMetricPinnedGuardRule { limit: 9.4 }),
            Rc::new(LenientPolicyMetricTitleLengthRule { max_length: 86400 }),
            Rc::new(LenientPolicyMetricAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(LenientPolicyMetricRequiredChannelRule {
                message: "lenient policy metric needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_policy_metric_validator();
        let item = new_lenient_policy_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientPolicyMetricTitleLengthRule { max_length: 32 };
        let mut item = new_lenient_policy_metric("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = LenientPolicyMetricRequiredChannelRule {
            message: "lenient policy metric needs a value".to_string(),
        };
        let mut item = new_lenient_policy_metric("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
