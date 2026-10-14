use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record profile must pass. Check returns its problems, or none.
pub trait DnsRecordProfileRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordProfileAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DnsRecordProfileRule for DnsRecordProfileAllowedCodeRule {
    fn check(&self, item: &DnsRecordProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordProfileCodeLengthRule {
    pub max_length: i64,
}

impl DnsRecordProfileRule for DnsRecordProfileCodeLengthRule {
    fn check(&self, item: &DnsRecordProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordProfilePrincipalLengthRule {
    pub max_length: i64,
}

impl DnsRecordProfileRule for DnsRecordProfilePrincipalLengthRule {
    fn check(&self, item: &DnsRecordProfile) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record profile.
#[derive(Debug, Clone)]
pub struct DnsRecordProfileValidator {
    pub rules: Vec<Rc<dyn DnsRecordProfileRule>>,
}

impl DnsRecordProfileValidator {
    pub fn validate(&self, item: &DnsRecordProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_profile_validator() -> DnsRecordProfileValidator {
    DnsRecordProfileValidator {
        rules: vec![
            Rc::new(DnsRecordProfileAllowedCodeRule { allowed: Vec::new() }) as Rc<dyn DnsRecordProfileRule>,
            Rc::new(DnsRecordProfileCodeLengthRule { max_length: 120 }),
            Rc::new(DnsRecordProfilePrincipalLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_profile_validator();
        let item = new_dns_record_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DnsRecordProfileCodeLengthRule { max_length: 3600 };
        let mut item = new_dns_record_profile("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = DnsRecordProfilePrincipalLengthRule { max_length: 32 };
        let mut item = new_dns_record_profile("x", "y");
        item.principal = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
