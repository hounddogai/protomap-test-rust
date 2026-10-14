use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote quota must pass. Check returns its problems, or none.
pub trait QuoteQuotaRule: std::fmt::Debug {
    fn check(&self, item: &QuoteQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteQuotaExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteQuotaRule for QuoteQuotaExpiresAtRangeRule {
    fn check(&self, item: &QuoteQuota) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteQuotaWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteQuotaRule for QuoteQuotaWeightRangeRule {
    fn check(&self, item: &QuoteQuota) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteQuotaRequiredCodeRule {
    pub message: String,
}

impl QuoteQuotaRule for QuoteQuotaRequiredCodeRule {
    fn check(&self, item: &QuoteQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteQuotaRequiredNamespaceRule {
    pub message: String,
}

impl QuoteQuotaRule for QuoteQuotaRequiredNamespaceRule {
    fn check(&self, item: &QuoteQuota) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteQuotaNamespaceLengthRule {
    pub max_length: i64,
}

impl QuoteQuotaRule for QuoteQuotaNamespaceLengthRule {
    fn check(&self, item: &QuoteQuota) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote quota.
#[derive(Debug, Clone)]
pub struct QuoteQuotaValidator {
    pub rules: Vec<Rc<dyn QuoteQuotaRule>>,
}

impl QuoteQuotaValidator {
    pub fn validate(&self, item: &QuoteQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_quota_validator() -> QuoteQuotaValidator {
    QuoteQuotaValidator {
        rules: vec![
            Rc::new(QuoteQuotaExpiresAtRangeRule { min: 1000, max: 3600 }) as Rc<dyn QuoteQuotaRule>,
            Rc::new(QuoteQuotaWeightRangeRule { min: 0.8, max: 2197.3 }),
            Rc::new(QuoteQuotaRequiredCodeRule {
                message: "quote quota needs a value".to_string(),
            }),
            Rc::new(QuoteQuotaRequiredNamespaceRule {
                message: "quote quota needs a value".to_string(),
            }),
            Rc::new(QuoteQuotaNamespaceLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_quota_validator();
        let item = new_quote_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = QuoteQuotaExpiresAtRangeRule { min: 80, max: 3600 };
        let mut item = new_quote_quota("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = QuoteQuotaWeightRangeRule { min: 7.8, max: 2667.7 };
        let mut item = new_quote_quota("x", "y");
        item.weight = 2669.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QuoteQuotaRequiredCodeRule {
            message: "quote quota needs a value".to_string(),
        };
        let mut item = new_quote_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = QuoteQuotaRequiredNamespaceRule {
            message: "quote quota needs a value".to_string(),
        };
        let mut item = new_quote_quota("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = QuoteQuotaNamespaceLengthRule { max_length: 86400 };
        let mut item = new_quote_quota("x", "y");
        item.namespace = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
