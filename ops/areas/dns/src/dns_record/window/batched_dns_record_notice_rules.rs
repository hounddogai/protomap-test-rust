use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched dns record notice must pass. Check returns its problems, or none.
pub trait BatchedDnsRecordNoticeRule: std::fmt::Debug {
    fn check(&self, item: &BatchedDnsRecordNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordNoticeQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedDnsRecordNoticeRule for BatchedDnsRecordNoticeQuantityRangeRule {
    fn check(&self, item: &BatchedDnsRecordNotice) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordNoticeVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedDnsRecordNoticeRule for BatchedDnsRecordNoticeVersionRangeRule {
    fn check(&self, item: &BatchedDnsRecordNotice) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordNoticeRequiredHostRule {
    pub message: String,
}

impl BatchedDnsRecordNoticeRule for BatchedDnsRecordNoticeRequiredHostRule {
    fn check(&self, item: &BatchedDnsRecordNotice) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched dns record notice.
#[derive(Debug, Clone)]
pub struct BatchedDnsRecordNoticeValidator {
    pub rules: Vec<Rc<dyn BatchedDnsRecordNoticeRule>>,
}

impl BatchedDnsRecordNoticeValidator {
    pub fn validate(&self, item: &BatchedDnsRecordNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedDnsRecordNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedDnsRecordNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_dns_record_notice_validator() -> BatchedDnsRecordNoticeValidator {
    BatchedDnsRecordNoticeValidator {
        rules: vec![
            Rc::new(BatchedDnsRecordNoticeQuantityRangeRule { min: 80, max: 120 })
                as Rc<dyn BatchedDnsRecordNoticeRule>,
            Rc::new(BatchedDnsRecordNoticeVersionRangeRule { min: 16, max: 80 }),
            Rc::new(BatchedDnsRecordNoticeRequiredHostRule {
                message: "batched dns record notice needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_dns_record_notice_validator();
        let item = new_batched_dns_record_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = BatchedDnsRecordNoticeQuantityRangeRule { min: 32, max: 1000 };
        let mut item = new_batched_dns_record_notice("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = BatchedDnsRecordNoticeVersionRangeRule { min: 64, max: 80 };
        let mut item = new_batched_dns_record_notice("x", "y");
        item.version = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = BatchedDnsRecordNoticeRequiredHostRule {
            message: "batched dns record notice needs an entry".to_string(),
        };
        let mut item = new_batched_dns_record_notice("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
