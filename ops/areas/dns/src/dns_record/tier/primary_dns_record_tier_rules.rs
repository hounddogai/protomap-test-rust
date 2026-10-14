use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary dns record tier must pass. Check returns its problems, or none.
pub trait PrimaryDnsRecordTierRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryDnsRecordTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordTierHostLengthRule {
    pub max_length: i64,
}

impl PrimaryDnsRecordTierRule for PrimaryDnsRecordTierHostLengthRule {
    fn check(&self, item: &PrimaryDnsRecordTier) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordTierRequiredHostRule {
    pub message: String,
}

impl PrimaryDnsRecordTierRule for PrimaryDnsRecordTierRequiredHostRule {
    fn check(&self, item: &PrimaryDnsRecordTier) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordTierIssuerLengthRule {
    pub max_length: i64,
}

impl PrimaryDnsRecordTierRule for PrimaryDnsRecordTierIssuerLengthRule {
    fn check(&self, item: &PrimaryDnsRecordTier) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary dns record tier.
#[derive(Debug, Clone)]
pub struct PrimaryDnsRecordTierValidator {
    pub rules: Vec<Rc<dyn PrimaryDnsRecordTierRule>>,
}

impl PrimaryDnsRecordTierValidator {
    pub fn validate(&self, item: &PrimaryDnsRecordTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryDnsRecordTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryDnsRecordTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_dns_record_tier_validator() -> PrimaryDnsRecordTierValidator {
    PrimaryDnsRecordTierValidator {
        rules: vec![
            Rc::new(PrimaryDnsRecordTierHostLengthRule { max_length: 32 }) as Rc<dyn PrimaryDnsRecordTierRule>,
            Rc::new(PrimaryDnsRecordTierRequiredHostRule {
                message: "primary dns record tier needs this field".to_string(),
            }),
            Rc::new(PrimaryDnsRecordTierIssuerLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_dns_record_tier_validator();
        let item = new_primary_dns_record_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = PrimaryDnsRecordTierHostLengthRule { max_length: 3600 };
        let mut item = new_primary_dns_record_tier("x", "y");
        item.host = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = PrimaryDnsRecordTierRequiredHostRule {
            message: "primary dns record tier needs a value".to_string(),
        };
        let mut item = new_primary_dns_record_tier("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = PrimaryDnsRecordTierIssuerLengthRule { max_length: 3600 };
        let mut item = new_primary_dns_record_tier("x", "y");
        item.issuer = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
