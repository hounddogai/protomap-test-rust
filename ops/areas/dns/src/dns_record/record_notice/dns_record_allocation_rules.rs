use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record allocation must pass. Check returns its problems, or none.
pub trait DnsRecordAllocationRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAllocationAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordAllocationRule for DnsRecordAllocationAmountRangeRule {
    fn check(&self, item: &DnsRecordAllocation) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAllocationAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl DnsRecordAllocationRule for DnsRecordAllocationAllowedSourceRule {
    fn check(&self, item: &DnsRecordAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is duplicated", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAllocationRequiredCodeRule {
    pub message: String,
}

impl DnsRecordAllocationRule for DnsRecordAllocationRequiredCodeRule {
    fn check(&self, item: &DnsRecordAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record allocation.
#[derive(Debug, Clone)]
pub struct DnsRecordAllocationValidator {
    pub rules: Vec<Rc<dyn DnsRecordAllocationRule>>,
}

impl DnsRecordAllocationValidator {
    pub fn validate(&self, item: &DnsRecordAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_allocation_validator() -> DnsRecordAllocationValidator {
    DnsRecordAllocationValidator {
        rules: vec![
            Rc::new(DnsRecordAllocationAmountRangeRule { min: 6.7, max: 298.0 }) as Rc<dyn DnsRecordAllocationRule>,
            Rc::new(DnsRecordAllocationAllowedSourceRule {
                allowed: vec!["seasonal".to_string(), "primary".to_string(), "fallback".to_string()],
            }),
            Rc::new(DnsRecordAllocationRequiredCodeRule {
                message: "dns record allocation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_allocation_validator();
        let item = new_dns_record_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = DnsRecordAllocationAmountRangeRule { min: 7.7, max: 4693.7 };
        let mut item = new_dns_record_allocation("x", "y");
        item.amount = 4695.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DnsRecordAllocationRequiredCodeRule {
            message: "dns record allocation needs an entry".to_string(),
        };
        let mut item = new_dns_record_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
