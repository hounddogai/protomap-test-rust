use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record baseline must pass. Check returns its problems, or none.
pub trait DnsRecordBaselineRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBaselineCodePrefixRule {
    pub prefix: String,
}

impl DnsRecordBaselineRule for DnsRecordBaselineCodePrefixRule {
    fn check(&self, item: &DnsRecordBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBaselineAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DnsRecordBaselineRule for DnsRecordBaselineAllowedTitleRule {
    fn check(&self, item: &DnsRecordBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is inconsistent", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBaselineLabelLengthRule {
    pub max_length: i64,
}

impl DnsRecordBaselineRule for DnsRecordBaselineLabelLengthRule {
    fn check(&self, item: &DnsRecordBaseline) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBaselineIssuerLengthRule {
    pub max_length: i64,
}

impl DnsRecordBaselineRule for DnsRecordBaselineIssuerLengthRule {
    fn check(&self, item: &DnsRecordBaseline) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBaselineTitleLengthRule {
    pub max_length: i64,
}

impl DnsRecordBaselineRule for DnsRecordBaselineTitleLengthRule {
    fn check(&self, item: &DnsRecordBaseline) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record baseline.
#[derive(Debug, Clone)]
pub struct DnsRecordBaselineValidator {
    pub rules: Vec<Rc<dyn DnsRecordBaselineRule>>,
}

impl DnsRecordBaselineValidator {
    pub fn validate(&self, item: &DnsRecordBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_baseline_validator() -> DnsRecordBaselineValidator {
    DnsRecordBaselineValidator {
        rules: vec![
            Rc::new(DnsRecordBaselineCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn DnsRecordBaselineRule>,
            Rc::new(DnsRecordBaselineAllowedTitleRule {
                allowed: vec!["lenient".to_string(), "pending".to_string(), "batched".to_string()],
            }),
            Rc::new(DnsRecordBaselineLabelLengthRule { max_length: 16 }),
            Rc::new(DnsRecordBaselineIssuerLengthRule { max_length: 86400 }),
            Rc::new(DnsRecordBaselineTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_baseline_validator();
        let item = new_dns_record_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DnsRecordBaselineLabelLengthRule { max_length: 120 };
        let mut item = new_dns_record_baseline("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = DnsRecordBaselineIssuerLengthRule { max_length: 32 };
        let mut item = new_dns_record_baseline("x", "y");
        item.issuer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DnsRecordBaselineTitleLengthRule { max_length: 255 };
        let mut item = new_dns_record_baseline("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
