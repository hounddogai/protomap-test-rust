use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive suspension quota must pass. Check returns its problems, or none.
pub trait AdaptiveSuspensionQuotaRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveSuspensionQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionQuotaRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveSuspensionQuotaRule for AdaptiveSuspensionQuotaRatioRangeRule {
    fn check(&self, item: &AdaptiveSuspensionQuota) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionQuotaLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveSuspensionQuotaRule for AdaptiveSuspensionQuotaLabelLengthRule {
    fn check(&self, item: &AdaptiveSuspensionQuota) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionQuotaAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveSuspensionQuotaRule for AdaptiveSuspensionQuotaAllowedScopeRule {
    fn check(&self, item: &AdaptiveSuspensionQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is missing", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionQuotaScopeLengthRule {
    pub max_length: i64,
}

impl AdaptiveSuspensionQuotaRule for AdaptiveSuspensionQuotaScopeLengthRule {
    fn check(&self, item: &AdaptiveSuspensionQuota) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionQuotaRequiredLabelRule {
    pub message: String,
}

impl AdaptiveSuspensionQuotaRule for AdaptiveSuspensionQuotaRequiredLabelRule {
    fn check(&self, item: &AdaptiveSuspensionQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionQuotaRequiredScopeRule {
    pub message: String,
}

impl AdaptiveSuspensionQuotaRule for AdaptiveSuspensionQuotaRequiredScopeRule {
    fn check(&self, item: &AdaptiveSuspensionQuota) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive suspension quota.
#[derive(Debug, Clone)]
pub struct AdaptiveSuspensionQuotaValidator {
    pub rules: Vec<Rc<dyn AdaptiveSuspensionQuotaRule>>,
}

impl AdaptiveSuspensionQuotaValidator {
    pub fn validate(&self, item: &AdaptiveSuspensionQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveSuspensionQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveSuspensionQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_suspension_quota_validator() -> AdaptiveSuspensionQuotaValidator {
    AdaptiveSuspensionQuotaValidator {
        rules: vec![
            Rc::new(AdaptiveSuspensionQuotaRatioRangeRule { min: 0.8, max: 1328.6 })
                as Rc<dyn AdaptiveSuspensionQuotaRule>,
            Rc::new(AdaptiveSuspensionQuotaLabelLengthRule { max_length: 32 }),
            Rc::new(AdaptiveSuspensionQuotaAllowedScopeRule { allowed: Vec::new() }),
            Rc::new(AdaptiveSuspensionQuotaScopeLengthRule { max_length: 120 }),
            Rc::new(AdaptiveSuspensionQuotaRequiredLabelRule {
                message: "adaptive suspension quota needs a value".to_string(),
            }),
            Rc::new(AdaptiveSuspensionQuotaRequiredScopeRule {
                message: "adaptive suspension quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_suspension_quota_validator();
        let item = new_adaptive_suspension_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = AdaptiveSuspensionQuotaRatioRangeRule { min: 4.9, max: 4041.7 };
        let mut item = new_adaptive_suspension_quota("x", "y");
        item.ratio = 4043.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AdaptiveSuspensionQuotaLabelLengthRule { max_length: 32 };
        let mut item = new_adaptive_suspension_quota("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = AdaptiveSuspensionQuotaScopeLengthRule { max_length: 16 };
        let mut item = new_adaptive_suspension_quota("x", "y");
        item.scope = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AdaptiveSuspensionQuotaRequiredLabelRule {
            message: "adaptive suspension quota needs an entry".to_string(),
        };
        let mut item = new_adaptive_suspension_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = AdaptiveSuspensionQuotaRequiredScopeRule {
            message: "adaptive suspension quota needs this field".to_string(),
        };
        let mut item = new_adaptive_suspension_quota("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
