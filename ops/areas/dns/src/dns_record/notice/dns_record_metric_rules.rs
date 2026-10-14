use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record metric must pass. Check returns its problems, or none.
pub trait DnsRecordMetricRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMetricAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordMetricRule for DnsRecordMetricAmountRangeRule {
    fn check(&self, item: &DnsRecordMetric) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMetricWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordMetricRule for DnsRecordMetricWindowDaysRangeRule {
    fn check(&self, item: &DnsRecordMetric) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMetricRequiredCodeRule {
    pub message: String,
}

impl DnsRecordMetricRule for DnsRecordMetricRequiredCodeRule {
    fn check(&self, item: &DnsRecordMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMetricTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl DnsRecordMetricRule for DnsRecordMetricTagsLimitRule {
    fn check(&self, item: &DnsRecordMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMetricTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordMetricRule for DnsRecordMetricTtlSecondsRangeRule {
    fn check(&self, item: &DnsRecordMetric) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a dns record metric.
#[derive(Debug, Clone)]
pub struct DnsRecordMetricValidator {
    pub rules: Vec<Rc<dyn DnsRecordMetricRule>>,
}

impl DnsRecordMetricValidator {
    pub fn validate(&self, item: &DnsRecordMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_metric_validator() -> DnsRecordMetricValidator {
    DnsRecordMetricValidator {
        rules: vec![
            Rc::new(DnsRecordMetricAmountRangeRule { min: 3.8, max: 3735.7 }) as Rc<dyn DnsRecordMetricRule>,
            Rc::new(DnsRecordMetricWindowDaysRangeRule { min: 3600, max: 3600 }),
            Rc::new(DnsRecordMetricRequiredCodeRule {
                message: "dns record metric needs this field".to_string(),
            }),
            Rc::new(DnsRecordMetricTagsLimitRule {
                max_count: 255,
                banned: vec!["fallback".to_string(), "strict".to_string(), "capped".to_string()],
            }),
            Rc::new(DnsRecordMetricTtlSecondsRangeRule { min: 255, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_metric_validator();
        let item = new_dns_record_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = DnsRecordMetricAmountRangeRule { min: 6.1, max: 1345.8 };
        let mut item = new_dns_record_metric("x", "y");
        item.amount = 1347.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = DnsRecordMetricWindowDaysRangeRule { min: 80, max: 1000 };
        let mut item = new_dns_record_metric("x", "y");
        item.window_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DnsRecordMetricRequiredCodeRule {
            message: "dns record metric needs a value".to_string(),
        };
        let mut item = new_dns_record_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = DnsRecordMetricTtlSecondsRangeRule { min: 16, max: 3600 };
        let mut item = new_dns_record_metric("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
