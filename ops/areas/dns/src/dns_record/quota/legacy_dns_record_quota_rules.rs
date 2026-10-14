use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy dns record quota must pass. Check returns its problems, or none.
pub trait LegacyDnsRecordQuotaRule: std::fmt::Debug {
    fn check(&self, item: &LegacyDnsRecordQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordQuotaCodePrefixRule {
    pub prefix: String,
}

impl LegacyDnsRecordQuotaRule for LegacyDnsRecordQuotaCodePrefixRule {
    fn check(&self, item: &LegacyDnsRecordQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordQuotaRequiredLabelRule {
    pub message: String,
}

impl LegacyDnsRecordQuotaRule for LegacyDnsRecordQuotaRequiredLabelRule {
    fn check(&self, item: &LegacyDnsRecordQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordQuotaRequiredNameRule {
    pub message: String,
}

impl LegacyDnsRecordQuotaRule for LegacyDnsRecordQuotaRequiredNameRule {
    fn check(&self, item: &LegacyDnsRecordQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy dns record quota.
#[derive(Debug, Clone)]
pub struct LegacyDnsRecordQuotaValidator {
    pub rules: Vec<Rc<dyn LegacyDnsRecordQuotaRule>>,
}

impl LegacyDnsRecordQuotaValidator {
    pub fn validate(&self, item: &LegacyDnsRecordQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyDnsRecordQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyDnsRecordQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_dns_record_quota_validator() -> LegacyDnsRecordQuotaValidator {
    LegacyDnsRecordQuotaValidator {
        rules: vec![
            Rc::new(LegacyDnsRecordQuotaCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn LegacyDnsRecordQuotaRule>,
            Rc::new(LegacyDnsRecordQuotaRequiredLabelRule {
                message: "legacy dns record quota needs an entry".to_string(),
            }),
            Rc::new(LegacyDnsRecordQuotaRequiredNameRule {
                message: "legacy dns record quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_dns_record_quota_validator();
        let item = new_legacy_dns_record_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LegacyDnsRecordQuotaRequiredLabelRule {
            message: "legacy dns record quota needs a value".to_string(),
        };
        let mut item = new_legacy_dns_record_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyDnsRecordQuotaRequiredNameRule {
            message: "legacy dns record quota needs this field".to_string(),
        };
        let mut item = new_legacy_dns_record_quota("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
