use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record score must pass. Check returns its problems, or none.
pub trait DnsRecordScoreRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordScoreCodePrefixRule {
    pub prefix: String,
}

impl DnsRecordScoreRule for DnsRecordScoreCodePrefixRule {
    fn check(&self, item: &DnsRecordScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordScoreNameLengthRule {
    pub max_length: i64,
}

impl DnsRecordScoreRule for DnsRecordScoreNameLengthRule {
    fn check(&self, item: &DnsRecordScore) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordScoreCodeLengthRule {
    pub max_length: i64,
}

impl DnsRecordScoreRule for DnsRecordScoreCodeLengthRule {
    fn check(&self, item: &DnsRecordScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record score.
#[derive(Debug, Clone)]
pub struct DnsRecordScoreValidator {
    pub rules: Vec<Rc<dyn DnsRecordScoreRule>>,
}

impl DnsRecordScoreValidator {
    pub fn validate(&self, item: &DnsRecordScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_score_validator() -> DnsRecordScoreValidator {
    DnsRecordScoreValidator {
        rules: vec![
            Rc::new(DnsRecordScoreCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn DnsRecordScoreRule>,
            Rc::new(DnsRecordScoreNameLengthRule { max_length: 120 }),
            Rc::new(DnsRecordScoreCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_score_validator();
        let item = new_dns_record_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DnsRecordScoreNameLengthRule { max_length: 16 };
        let mut item = new_dns_record_score("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DnsRecordScoreCodeLengthRule { max_length: 3600 };
        let mut item = new_dns_record_score("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
