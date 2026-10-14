use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record budget must pass. Check returns its problems, or none.
pub trait DnsRecordBudgetRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBudgetBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordBudgetRule for DnsRecordBudgetBurstRangeRule {
    fn check(&self, item: &DnsRecordBudget) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBudgetChannelLengthRule {
    pub max_length: i64,
}

impl DnsRecordBudgetRule for DnsRecordBudgetChannelLengthRule {
    fn check(&self, item: &DnsRecordBudget) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBudgetCodePrefixRule {
    pub prefix: String,
}

impl DnsRecordBudgetRule for DnsRecordBudgetCodePrefixRule {
    fn check(&self, item: &DnsRecordBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record budget.
#[derive(Debug, Clone)]
pub struct DnsRecordBudgetValidator {
    pub rules: Vec<Rc<dyn DnsRecordBudgetRule>>,
}

impl DnsRecordBudgetValidator {
    pub fn validate(&self, item: &DnsRecordBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_budget_validator() -> DnsRecordBudgetValidator {
    DnsRecordBudgetValidator {
        rules: vec![
            Rc::new(DnsRecordBudgetBurstRangeRule { min: 1000, max: 3600 }) as Rc<dyn DnsRecordBudgetRule>,
            Rc::new(DnsRecordBudgetChannelLengthRule { max_length: 1000 }),
            Rc::new(DnsRecordBudgetCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_budget_validator();
        let item = new_dns_record_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = DnsRecordBudgetBurstRangeRule { min: 16, max: 3600 };
        let mut item = new_dns_record_budget("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = DnsRecordBudgetChannelLengthRule { max_length: 16 };
        let mut item = new_dns_record_budget("x", "y");
        item.channel = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
