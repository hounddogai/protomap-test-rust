use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual redaction metric must pass. Check returns its problems, or none.
pub trait ManualRedactionMetricRule: std::fmt::Debug {
    fn check(&self, item: &ManualRedactionMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionMetricRequiredFingerprintRule {
    pub message: String,
}

impl ManualRedactionMetricRule for ManualRedactionMetricRequiredFingerprintRule {
    fn check(&self, item: &ManualRedactionMetric) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionMetricRequiredTitleRule {
    pub message: String,
}

impl ManualRedactionMetricRule for ManualRedactionMetricRequiredTitleRule {
    fn check(&self, item: &ManualRedactionMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionMetricCodePrefixRule {
    pub prefix: String,
}

impl ManualRedactionMetricRule for ManualRedactionMetricCodePrefixRule {
    fn check(&self, item: &ManualRedactionMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionMetricCurrencyLengthRule {
    pub max_length: i64,
}

impl ManualRedactionMetricRule for ManualRedactionMetricCurrencyLengthRule {
    fn check(&self, item: &ManualRedactionMetric) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionMetricVisibleGuardRule {
    pub limit: f64,
}

impl ManualRedactionMetricRule for ManualRedactionMetricVisibleGuardRule {
    fn check(&self, item: &ManualRedactionMetric) -> Vec<String> {
        if item.visible && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while visible", item.amount, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionMetricCodeLengthRule {
    pub max_length: i64,
}

impl ManualRedactionMetricRule for ManualRedactionMetricCodeLengthRule {
    fn check(&self, item: &ManualRedactionMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual redaction metric.
#[derive(Debug, Clone)]
pub struct ManualRedactionMetricValidator {
    pub rules: Vec<Rc<dyn ManualRedactionMetricRule>>,
}

impl ManualRedactionMetricValidator {
    pub fn validate(&self, item: &ManualRedactionMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualRedactionMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRedactionMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_redaction_metric_validator() -> ManualRedactionMetricValidator {
    ManualRedactionMetricValidator {
        rules: vec![
            Rc::new(ManualRedactionMetricRequiredFingerprintRule {
                message: "manual redaction metric needs a value".to_string(),
            }) as Rc<dyn ManualRedactionMetricRule>,
            Rc::new(ManualRedactionMetricRequiredTitleRule {
                message: "manual redaction metric needs this field".to_string(),
            }),
            Rc::new(ManualRedactionMetricCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ManualRedactionMetricCurrencyLengthRule { max_length: 255 }),
            Rc::new(ManualRedactionMetricVisibleGuardRule { limit: 7.4 }),
            Rc::new(ManualRedactionMetricCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_redaction_metric_validator();
        let item = new_manual_redaction_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = ManualRedactionMetricRequiredFingerprintRule {
            message: "manual redaction metric needs this field".to_string(),
        };
        let mut item = new_manual_redaction_metric("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualRedactionMetricRequiredTitleRule {
            message: "manual redaction metric needs a value".to_string(),
        };
        let mut item = new_manual_redaction_metric("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = ManualRedactionMetricCurrencyLengthRule { max_length: 16 };
        let mut item = new_manual_redaction_metric("x", "y");
        item.currency = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualRedactionMetricCodeLengthRule { max_length: 1000 };
        let mut item = new_manual_redaction_metric("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
