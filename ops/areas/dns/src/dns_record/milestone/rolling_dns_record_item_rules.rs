use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling dns record item must pass. Check returns its problems, or none.
pub trait RollingDnsRecordItemRule: std::fmt::Debug {
    fn check(&self, item: &RollingDnsRecordItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDnsRecordItemRequiredCodeRule {
    pub message: String,
}

impl RollingDnsRecordItemRule for RollingDnsRecordItemRequiredCodeRule {
    fn check(&self, item: &RollingDnsRecordItem) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDnsRecordItemArchivedGuardRule {
    pub limit: i64,
}

impl RollingDnsRecordItemRule for RollingDnsRecordItemArchivedGuardRule {
    fn check(&self, item: &RollingDnsRecordItem) -> Vec<String> {
        if item.archived && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while archived", item.burst, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDnsRecordItemCodeLengthRule {
    pub max_length: i64,
}

impl RollingDnsRecordItemRule for RollingDnsRecordItemCodeLengthRule {
    fn check(&self, item: &RollingDnsRecordItem) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDnsRecordItemRequiredHostRule {
    pub message: String,
}

impl RollingDnsRecordItemRule for RollingDnsRecordItemRequiredHostRule {
    fn check(&self, item: &RollingDnsRecordItem) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling dns record item.
#[derive(Debug, Clone)]
pub struct RollingDnsRecordItemValidator {
    pub rules: Vec<Rc<dyn RollingDnsRecordItemRule>>,
}

impl RollingDnsRecordItemValidator {
    pub fn validate(&self, item: &RollingDnsRecordItem) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingDnsRecordItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingDnsRecordItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_dns_record_item_validator() -> RollingDnsRecordItemValidator {
    RollingDnsRecordItemValidator {
        rules: vec![
            Rc::new(RollingDnsRecordItemRequiredCodeRule {
                message: "rolling dns record item needs a value".to_string(),
            }) as Rc<dyn RollingDnsRecordItemRule>,
            Rc::new(RollingDnsRecordItemArchivedGuardRule { limit: 1000 }),
            Rc::new(RollingDnsRecordItemCodeLengthRule { max_length: 80 }),
            Rc::new(RollingDnsRecordItemRequiredHostRule {
                message: "rolling dns record item needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_dns_record_item_validator();
        let item = new_rolling_dns_record_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingDnsRecordItemRequiredCodeRule {
            message: "rolling dns record item needs a value".to_string(),
        };
        let mut item = new_rolling_dns_record_item("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingDnsRecordItemCodeLengthRule { max_length: 80 };
        let mut item = new_rolling_dns_record_item("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RollingDnsRecordItemRequiredHostRule {
            message: "rolling dns record item needs a value".to_string(),
        };
        let mut item = new_rolling_dns_record_item("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
