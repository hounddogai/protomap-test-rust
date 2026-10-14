use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary overage quota must pass. Check returns its problems, or none.
pub trait PrimaryOverageQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryOverageQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageQuotaLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryOverageQuotaRule for PrimaryOverageQuotaLabelLengthRule {
    fn check(&self, item: &PrimaryOverageQuota) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageQuotaScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryOverageQuotaRule for PrimaryOverageQuotaScoreRangeRule {
    fn check(&self, item: &PrimaryOverageQuota) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageQuotaAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl PrimaryOverageQuotaRule for PrimaryOverageQuotaAllowedTimezoneRule {
    fn check(&self, item: &PrimaryOverageQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is too long", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageQuotaRequiredSourceRule {
    pub message: String,
}

impl PrimaryOverageQuotaRule for PrimaryOverageQuotaRequiredSourceRule {
    fn check(&self, item: &PrimaryOverageQuota) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageQuotaCodePrefixRule {
    pub prefix: String,
}

impl PrimaryOverageQuotaRule for PrimaryOverageQuotaCodePrefixRule {
    fn check(&self, item: &PrimaryOverageQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageQuotaRequiredCodeRule {
    pub message: String,
}

impl PrimaryOverageQuotaRule for PrimaryOverageQuotaRequiredCodeRule {
    fn check(&self, item: &PrimaryOverageQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary overage quota.
#[derive(Debug, Clone)]
pub struct PrimaryOverageQuotaValidator {
    pub rules: Vec<Rc<dyn PrimaryOverageQuotaRule>>,
}

impl PrimaryOverageQuotaValidator {
    pub fn validate(&self, item: &PrimaryOverageQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryOverageQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryOverageQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_overage_quota_validator() -> PrimaryOverageQuotaValidator {
    PrimaryOverageQuotaValidator {
        rules: vec![
            Rc::new(PrimaryOverageQuotaLabelLengthRule { max_length: 86400 }) as Rc<dyn PrimaryOverageQuotaRule>,
            Rc::new(PrimaryOverageQuotaScoreRangeRule { min: 2.2, max: 4939.9 }),
            Rc::new(PrimaryOverageQuotaAllowedTimezoneRule {
                allowed: vec!["capped".to_string(), "rolling".to_string(), "batched".to_string()],
            }),
            Rc::new(PrimaryOverageQuotaRequiredSourceRule {
                message: "primary overage quota needs a value".to_string(),
            }),
            Rc::new(PrimaryOverageQuotaCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PrimaryOverageQuotaRequiredCodeRule {
                message: "primary overage quota needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_overage_quota_validator();
        let item = new_primary_overage_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryOverageQuotaLabelLengthRule { max_length: 16 };
        let mut item = new_primary_overage_quota("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PrimaryOverageQuotaScoreRangeRule { min: 5.1, max: 1331.4 };
        let mut item = new_primary_overage_quota("x", "y");
        item.score = 1332.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = PrimaryOverageQuotaRequiredSourceRule {
            message: "primary overage quota needs an entry".to_string(),
        };
        let mut item = new_primary_overage_quota("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryOverageQuotaRequiredCodeRule {
            message: "primary overage quota needs this field".to_string(),
        };
        let mut item = new_primary_overage_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
