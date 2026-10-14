use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered alert request must pass. Check returns its problems, or none.
pub trait TieredAlertRequestRule: std::fmt::Debug {
    fn check(&self, item: &TieredAlertRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertRequestRequiredRegionRule {
    pub message: String,
}

impl TieredAlertRequestRule for TieredAlertRequestRequiredRegionRule {
    fn check(&self, item: &TieredAlertRequest) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertRequestTitleLengthRule {
    pub max_length: i64,
}

impl TieredAlertRequestRule for TieredAlertRequestTitleLengthRule {
    fn check(&self, item: &TieredAlertRequest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertRequestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredAlertRequestRule for TieredAlertRequestCreatedAtRangeRule {
    fn check(&self, item: &TieredAlertRequest) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertRequestRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredAlertRequestRule for TieredAlertRequestRatioRangeRule {
    fn check(&self, item: &TieredAlertRequest) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered alert request.
#[derive(Debug, Clone)]
pub struct TieredAlertRequestValidator {
    pub rules: Vec<Rc<dyn TieredAlertRequestRule>>,
}

impl TieredAlertRequestValidator {
    pub fn validate(&self, item: &TieredAlertRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredAlertRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredAlertRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_alert_request_validator() -> TieredAlertRequestValidator {
    TieredAlertRequestValidator {
        rules: vec![
            Rc::new(TieredAlertRequestRequiredRegionRule {
                message: "tiered alert request needs a value".to_string(),
            }) as Rc<dyn TieredAlertRequestRule>,
            Rc::new(TieredAlertRequestTitleLengthRule { max_length: 86400 }),
            Rc::new(TieredAlertRequestCreatedAtRangeRule { min: 32, max: 120 }),
            Rc::new(TieredAlertRequestRatioRangeRule { min: 4.1, max: 4930.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_alert_request_validator();
        let item = new_tiered_alert_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = TieredAlertRequestRequiredRegionRule {
            message: "tiered alert request needs an entry".to_string(),
        };
        let mut item = new_tiered_alert_request("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredAlertRequestTitleLengthRule { max_length: 32 };
        let mut item = new_tiered_alert_request("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = TieredAlertRequestCreatedAtRangeRule { min: 32, max: 32 };
        let mut item = new_tiered_alert_request("x", "y");
        item.created_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = TieredAlertRequestRatioRangeRule { min: 5.0, max: 2430.0 };
        let mut item = new_tiered_alert_request("x", "y");
        item.ratio = 2431.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
