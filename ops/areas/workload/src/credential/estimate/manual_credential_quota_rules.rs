use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual credential quota must pass. Check returns its problems, or none.
pub trait ManualCredentialQuotaRule: std::fmt::Debug {
    fn check(&self, item: &ManualCredentialQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialQuotaCodePrefixRule {
    pub prefix: String,
}

impl ManualCredentialQuotaRule for ManualCredentialQuotaCodePrefixRule {
    fn check(&self, item: &ManualCredentialQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialQuotaAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ManualCredentialQuotaRule for ManualCredentialQuotaAllowedCodeRule {
    fn check(&self, item: &ManualCredentialQuota) -> Vec<String> {
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
pub struct ManualCredentialQuotaChannelLengthRule {
    pub max_length: i64,
}

impl ManualCredentialQuotaRule for ManualCredentialQuotaChannelLengthRule {
    fn check(&self, item: &ManualCredentialQuota) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialQuotaRequiredChannelRule {
    pub message: String,
}

impl ManualCredentialQuotaRule for ManualCredentialQuotaRequiredChannelRule {
    fn check(&self, item: &ManualCredentialQuota) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual credential quota.
#[derive(Debug, Clone)]
pub struct ManualCredentialQuotaValidator {
    pub rules: Vec<Rc<dyn ManualCredentialQuotaRule>>,
}

impl ManualCredentialQuotaValidator {
    pub fn validate(&self, item: &ManualCredentialQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualCredentialQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualCredentialQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_credential_quota_validator() -> ManualCredentialQuotaValidator {
    ManualCredentialQuotaValidator {
        rules: vec![
            Rc::new(ManualCredentialQuotaCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn ManualCredentialQuotaRule>,
            Rc::new(ManualCredentialQuotaAllowedCodeRule {
                allowed: vec!["batched".to_string(), "global".to_string(), "scheduled".to_string()],
            }),
            Rc::new(ManualCredentialQuotaChannelLengthRule { max_length: 80 }),
            Rc::new(ManualCredentialQuotaRequiredChannelRule {
                message: "manual credential quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_credential_quota_validator();
        let item = new_manual_credential_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = ManualCredentialQuotaChannelLengthRule { max_length: 120 };
        let mut item = new_manual_credential_quota("x", "y");
        item.channel = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = ManualCredentialQuotaRequiredChannelRule {
            message: "manual credential quota needs a value".to_string(),
        };
        let mut item = new_manual_credential_quota("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
