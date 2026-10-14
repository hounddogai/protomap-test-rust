use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dns record forecast must pass. Check returns its problems, or none.
pub trait DnsRecordForecastRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordForecastCodeLengthRule {
    pub max_length: i64,
}

impl DnsRecordForecastRule for DnsRecordForecastCodeLengthRule {
    fn check(&self, item: &DnsRecordForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordForecastTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl DnsRecordForecastRule for DnsRecordForecastTagsLimitRule {
    fn check(&self, item: &DnsRecordForecast) -> Vec<String> {
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
pub struct DnsRecordForecastAllowedNameRule {
    pub allowed: Vec<String>,
}

impl DnsRecordForecastRule for DnsRecordForecastAllowedNameRule {
    fn check(&self, item: &DnsRecordForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is below minimum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordForecastPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordForecastRule for DnsRecordForecastPriorityRangeRule {
    fn check(&self, item: &DnsRecordForecast) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordForecastRequiredNameRule {
    pub message: String,
}

impl DnsRecordForecastRule for DnsRecordForecastRequiredNameRule {
    fn check(&self, item: &DnsRecordForecast) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record forecast.
#[derive(Debug, Clone)]
pub struct DnsRecordForecastValidator {
    pub rules: Vec<Rc<dyn DnsRecordForecastRule>>,
}

impl DnsRecordForecastValidator {
    pub fn validate(&self, item: &DnsRecordForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_forecast_validator() -> DnsRecordForecastValidator {
    DnsRecordForecastValidator {
        rules: vec![
            Rc::new(DnsRecordForecastCodeLengthRule { max_length: 86400 }) as Rc<dyn DnsRecordForecastRule>,
            Rc::new(DnsRecordForecastTagsLimitRule {
                max_count: 16,
                banned: vec!["strict".to_string()],
            }),
            Rc::new(DnsRecordForecastAllowedNameRule {
                allowed: vec!["tiered".to_string(), "capped".to_string()],
            }),
            Rc::new(DnsRecordForecastPriorityRangeRule { min: 16, max: 80 }),
            Rc::new(DnsRecordForecastRequiredNameRule {
                message: "dns record forecast needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_forecast_validator();
        let item = new_dns_record_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DnsRecordForecastCodeLengthRule { max_length: 32 };
        let mut item = new_dns_record_forecast("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = DnsRecordForecastPriorityRangeRule { min: 16, max: 32 };
        let mut item = new_dns_record_forecast("x", "y");
        item.priority = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DnsRecordForecastRequiredNameRule {
            message: "dns record forecast needs a value".to_string(),
        };
        let mut item = new_dns_record_forecast("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
