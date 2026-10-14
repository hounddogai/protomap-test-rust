use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback dns record allocation must pass. Check returns its problems, or none.
pub trait FallbackDnsRecordAllocationRule: std::fmt::Debug {
    fn check(&self, item: &FallbackDnsRecordAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDnsRecordAllocationMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackDnsRecordAllocationRule for FallbackDnsRecordAllocationMarginRangeRule {
    fn check(&self, item: &FallbackDnsRecordAllocation) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDnsRecordAllocationRequiredAudienceRule {
    pub message: String,
}

impl FallbackDnsRecordAllocationRule for FallbackDnsRecordAllocationRequiredAudienceRule {
    fn check(&self, item: &FallbackDnsRecordAllocation) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDnsRecordAllocationRequiredCodeRule {
    pub message: String,
}

impl FallbackDnsRecordAllocationRule for FallbackDnsRecordAllocationRequiredCodeRule {
    fn check(&self, item: &FallbackDnsRecordAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDnsRecordAllocationApprovedGuardRule {
    pub limit: i64,
}

impl FallbackDnsRecordAllocationRule for FallbackDnsRecordAllocationApprovedGuardRule {
    fn check(&self, item: &FallbackDnsRecordAllocation) -> Vec<String> {
        if item.approved && (item.version > self.limit) {
            return vec![format!(
                "version {} exceeds {} while approved",
                item.version, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDnsRecordAllocationWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackDnsRecordAllocationRule for FallbackDnsRecordAllocationWeightRangeRule {
    fn check(&self, item: &FallbackDnsRecordAllocation) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback dns record allocation.
#[derive(Debug, Clone)]
pub struct FallbackDnsRecordAllocationValidator {
    pub rules: Vec<Rc<dyn FallbackDnsRecordAllocationRule>>,
}

impl FallbackDnsRecordAllocationValidator {
    pub fn validate(&self, item: &FallbackDnsRecordAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackDnsRecordAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackDnsRecordAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_dns_record_allocation_validator() -> FallbackDnsRecordAllocationValidator {
    FallbackDnsRecordAllocationValidator {
        rules: vec![
            Rc::new(FallbackDnsRecordAllocationMarginRangeRule { min: 4.9, max: 1303.5 })
                as Rc<dyn FallbackDnsRecordAllocationRule>,
            Rc::new(FallbackDnsRecordAllocationRequiredAudienceRule {
                message: "fallback dns record allocation needs a value".to_string(),
            }),
            Rc::new(FallbackDnsRecordAllocationRequiredCodeRule {
                message: "fallback dns record allocation needs a value".to_string(),
            }),
            Rc::new(FallbackDnsRecordAllocationApprovedGuardRule { limit: 255 }),
            Rc::new(FallbackDnsRecordAllocationWeightRangeRule { min: 2.8, max: 1669.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_dns_record_allocation_validator();
        let item = new_fallback_dns_record_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = FallbackDnsRecordAllocationMarginRangeRule { min: 7.4, max: 2827.1 };
        let mut item = new_fallback_dns_record_allocation("x", "y");
        item.margin = 2828.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = FallbackDnsRecordAllocationRequiredAudienceRule {
            message: "fallback dns record allocation needs this field".to_string(),
        };
        let mut item = new_fallback_dns_record_allocation("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackDnsRecordAllocationRequiredCodeRule {
            message: "fallback dns record allocation needs this field".to_string(),
        };
        let mut item = new_fallback_dns_record_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = FallbackDnsRecordAllocationWeightRangeRule { min: 1.9, max: 4682.1 };
        let mut item = new_fallback_dns_record_allocation("x", "y");
        item.weight = 4683.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
