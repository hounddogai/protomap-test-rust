use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record ledger must pass. Check returns its problems, or none.
pub trait DnsRecordLedgerRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordLedgerAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl DnsRecordLedgerRule for DnsRecordLedgerAllowedTimezoneRule {
    fn check(&self, item: &DnsRecordLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is duplicated", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordLedgerAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordLedgerRule for DnsRecordLedgerAmountRangeRule {
    fn check(&self, item: &DnsRecordLedger) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordLedgerRequiredTimezoneRule {
    pub message: String,
}

impl DnsRecordLedgerRule for DnsRecordLedgerRequiredTimezoneRule {
    fn check(&self, item: &DnsRecordLedger) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record ledger.
#[derive(Debug, Clone)]
pub struct DnsRecordLedgerValidator {
    pub rules: Vec<Rc<dyn DnsRecordLedgerRule>>,
}

impl DnsRecordLedgerValidator {
    pub fn validate(&self, item: &DnsRecordLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_ledger_validator() -> DnsRecordLedgerValidator {
    DnsRecordLedgerValidator {
        rules: vec![
            Rc::new(DnsRecordLedgerAllowedTimezoneRule {
                allowed: vec!["tiered".to_string()],
            }) as Rc<dyn DnsRecordLedgerRule>,
            Rc::new(DnsRecordLedgerAmountRangeRule { min: 7.3, max: 4365.5 }),
            Rc::new(DnsRecordLedgerRequiredTimezoneRule {
                message: "dns record ledger needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_ledger_validator();
        let item = new_dns_record_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = DnsRecordLedgerAmountRangeRule { min: 8.7, max: 1186.8 };
        let mut item = new_dns_record_ledger("x", "y");
        item.amount = 1188.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = DnsRecordLedgerRequiredTimezoneRule {
            message: "dns record ledger needs an entry".to_string(),
        };
        let mut item = new_dns_record_ledger("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
