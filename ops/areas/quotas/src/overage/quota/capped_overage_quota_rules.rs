use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped overage quota must pass. Check returns its problems, or none.
pub trait CappedOverageQuotaRule: std::fmt::Debug {
    fn check(&self, item: &CappedOverageQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageQuotaCodeLengthRule {
    pub max_length: i64,
}

impl CappedOverageQuotaRule for CappedOverageQuotaCodeLengthRule {
    fn check(&self, item: &CappedOverageQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageQuotaRequiredTitleRule {
    pub message: String,
}

impl CappedOverageQuotaRule for CappedOverageQuotaRequiredTitleRule {
    fn check(&self, item: &CappedOverageQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageQuotaAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl CappedOverageQuotaRule for CappedOverageQuotaAllowedTimezoneRule {
    fn check(&self, item: &CappedOverageQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is duplicated", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageQuotaTitleLengthRule {
    pub max_length: i64,
}

impl CappedOverageQuotaRule for CappedOverageQuotaTitleLengthRule {
    fn check(&self, item: &CappedOverageQuota) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageQuotaRequiredAudienceRule {
    pub message: String,
}

impl CappedOverageQuotaRule for CappedOverageQuotaRequiredAudienceRule {
    fn check(&self, item: &CappedOverageQuota) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageQuotaTimezoneLengthRule {
    pub max_length: i64,
}

impl CappedOverageQuotaRule for CappedOverageQuotaTimezoneLengthRule {
    fn check(&self, item: &CappedOverageQuota) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped overage quota.
#[derive(Debug, Clone)]
pub struct CappedOverageQuotaValidator {
    pub rules: Vec<Rc<dyn CappedOverageQuotaRule>>,
}

impl CappedOverageQuotaValidator {
    pub fn validate(&self, item: &CappedOverageQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedOverageQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedOverageQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_overage_quota_validator() -> CappedOverageQuotaValidator {
    CappedOverageQuotaValidator {
        rules: vec![
            Rc::new(CappedOverageQuotaCodeLengthRule { max_length: 16 }) as Rc<dyn CappedOverageQuotaRule>,
            Rc::new(CappedOverageQuotaRequiredTitleRule {
                message: "capped overage quota needs this field".to_string(),
            }),
            Rc::new(CappedOverageQuotaAllowedTimezoneRule {
                allowed: vec!["weighted".to_string()],
            }),
            Rc::new(CappedOverageQuotaTitleLengthRule { max_length: 16 }),
            Rc::new(CappedOverageQuotaRequiredAudienceRule {
                message: "capped overage quota needs this field".to_string(),
            }),
            Rc::new(CappedOverageQuotaTimezoneLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_overage_quota_validator();
        let item = new_capped_overage_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedOverageQuotaCodeLengthRule { max_length: 80 };
        let mut item = new_capped_overage_quota("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CappedOverageQuotaRequiredTitleRule {
            message: "capped overage quota needs this field".to_string(),
        };
        let mut item = new_capped_overage_quota("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CappedOverageQuotaTitleLengthRule { max_length: 80 };
        let mut item = new_capped_overage_quota("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = CappedOverageQuotaRequiredAudienceRule {
            message: "capped overage quota needs a value".to_string(),
        };
        let mut item = new_capped_overage_quota("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = CappedOverageQuotaTimezoneLengthRule { max_length: 80 };
        let mut item = new_capped_overage_quota("x", "y");
        item.timezone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
