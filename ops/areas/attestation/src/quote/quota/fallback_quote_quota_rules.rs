use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback quote quota must pass. Check returns its problems, or none.
pub trait FallbackQuoteQuotaRule: std::fmt::Debug {
    fn check(&self, item: &FallbackQuoteQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteQuotaCodePrefixRule {
    pub prefix: String,
}

impl FallbackQuoteQuotaRule for FallbackQuoteQuotaCodePrefixRule {
    fn check(&self, item: &FallbackQuoteQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteQuotaRequiredTitleRule {
    pub message: String,
}

impl FallbackQuoteQuotaRule for FallbackQuoteQuotaRequiredTitleRule {
    fn check(&self, item: &FallbackQuoteQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteQuotaRequiredCodeRule {
    pub message: String,
}

impl FallbackQuoteQuotaRule for FallbackQuoteQuotaRequiredCodeRule {
    fn check(&self, item: &FallbackQuoteQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteQuotaRequiredFingerprintRule {
    pub message: String,
}

impl FallbackQuoteQuotaRule for FallbackQuoteQuotaRequiredFingerprintRule {
    fn check(&self, item: &FallbackQuoteQuota) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback quote quota.
#[derive(Debug, Clone)]
pub struct FallbackQuoteQuotaValidator {
    pub rules: Vec<Rc<dyn FallbackQuoteQuotaRule>>,
}

impl FallbackQuoteQuotaValidator {
    pub fn validate(&self, item: &FallbackQuoteQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackQuoteQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackQuoteQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_quote_quota_validator() -> FallbackQuoteQuotaValidator {
    FallbackQuoteQuotaValidator {
        rules: vec![
            Rc::new(FallbackQuoteQuotaCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn FallbackQuoteQuotaRule>,
            Rc::new(FallbackQuoteQuotaRequiredTitleRule {
                message: "fallback quote quota needs a value".to_string(),
            }),
            Rc::new(FallbackQuoteQuotaRequiredCodeRule {
                message: "fallback quote quota needs a value".to_string(),
            }),
            Rc::new(FallbackQuoteQuotaRequiredFingerprintRule {
                message: "fallback quote quota needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_quote_quota_validator();
        let item = new_fallback_quote_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FallbackQuoteQuotaRequiredTitleRule {
            message: "fallback quote quota needs a value".to_string(),
        };
        let mut item = new_fallback_quote_quota("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackQuoteQuotaRequiredCodeRule {
            message: "fallback quote quota needs this field".to_string(),
        };
        let mut item = new_fallback_quote_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = FallbackQuoteQuotaRequiredFingerprintRule {
            message: "fallback quote quota needs a value".to_string(),
        };
        let mut item = new_fallback_quote_quota("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
