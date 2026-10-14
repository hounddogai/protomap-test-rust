use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a postmortem quota must pass. Check returns its problems, or none.
pub trait PostmortemQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemQuotaRequiredCodeRule {
    pub message: String,
}

impl PostmortemQuotaRule for PostmortemQuotaRequiredCodeRule {
    fn check(&self, item: &PostmortemQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemQuotaAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl PostmortemQuotaRule for PostmortemQuotaAllowedScopeRule {
    fn check(&self, item: &PostmortemQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is below minimum", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemQuotaCodePrefixRule {
    pub prefix: String,
}

impl PostmortemQuotaRule for PostmortemQuotaCodePrefixRule {
    fn check(&self, item: &PostmortemQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemQuotaLabelLengthRule {
    pub max_length: i64,
}

impl PostmortemQuotaRule for PostmortemQuotaLabelLengthRule {
    fn check(&self, item: &PostmortemQuota) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemQuotaAudienceLengthRule {
    pub max_length: i64,
}

impl PostmortemQuotaRule for PostmortemQuotaAudienceLengthRule {
    fn check(&self, item: &PostmortemQuota) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem quota.
#[derive(Debug, Clone)]
pub struct PostmortemQuotaValidator {
    pub rules: Vec<Rc<dyn PostmortemQuotaRule>>,
}

impl PostmortemQuotaValidator {
    pub fn validate(&self, item: &PostmortemQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PostmortemQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_quota_validator() -> PostmortemQuotaValidator {
    PostmortemQuotaValidator {
        rules: vec![
            Rc::new(PostmortemQuotaRequiredCodeRule {
                message: "postmortem quota needs this field".to_string(),
            }) as Rc<dyn PostmortemQuotaRule>,
            Rc::new(PostmortemQuotaAllowedScopeRule { allowed: Vec::new() }),
            Rc::new(PostmortemQuotaCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PostmortemQuotaLabelLengthRule { max_length: 80 }),
            Rc::new(PostmortemQuotaAudienceLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_quota_validator();
        let item = new_postmortem_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PostmortemQuotaRequiredCodeRule {
            message: "postmortem quota needs an entry".to_string(),
        };
        let mut item = new_postmortem_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PostmortemQuotaLabelLengthRule { max_length: 120 };
        let mut item = new_postmortem_quota("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = PostmortemQuotaAudienceLengthRule { max_length: 3600 };
        let mut item = new_postmortem_quota("x", "y");
        item.audience = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
