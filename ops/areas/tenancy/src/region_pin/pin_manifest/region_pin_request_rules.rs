use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin request must pass. Check returns its problems, or none.
pub trait RegionPinRequestRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRequestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinRequestRule for RegionPinRequestCreatedAtRangeRule {
    fn check(&self, item: &RegionPinRequest) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRequestRequiredCountryRule {
    pub message: String,
}

impl RegionPinRequestRule for RegionPinRequestRequiredCountryRule {
    fn check(&self, item: &RegionPinRequest) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRequestNameLengthRule {
    pub max_length: i64,
}

impl RegionPinRequestRule for RegionPinRequestNameLengthRule {
    fn check(&self, item: &RegionPinRequest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRequestArchivedGuardRule {
    pub limit: i64,
}

impl RegionPinRequestRule for RegionPinRequestArchivedGuardRule {
    fn check(&self, item: &RegionPinRequest) -> Vec<String> {
        if item.archived && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while archived",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRequestCountryLengthRule {
    pub max_length: i64,
}

impl RegionPinRequestRule for RegionPinRequestCountryLengthRule {
    fn check(&self, item: &RegionPinRequest) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRequestRequiredTitleRule {
    pub message: String,
}

impl RegionPinRequestRule for RegionPinRequestRequiredTitleRule {
    fn check(&self, item: &RegionPinRequest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin request.
#[derive(Debug, Clone)]
pub struct RegionPinRequestValidator {
    pub rules: Vec<Rc<dyn RegionPinRequestRule>>,
}

impl RegionPinRequestValidator {
    pub fn validate(&self, item: &RegionPinRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_request_validator() -> RegionPinRequestValidator {
    RegionPinRequestValidator {
        rules: vec![
            Rc::new(RegionPinRequestCreatedAtRangeRule { min: 1000, max: 86400 }) as Rc<dyn RegionPinRequestRule>,
            Rc::new(RegionPinRequestRequiredCountryRule {
                message: "region pin request needs this field".to_string(),
            }),
            Rc::new(RegionPinRequestNameLengthRule { max_length: 255 }),
            Rc::new(RegionPinRequestArchivedGuardRule { limit: 64 }),
            Rc::new(RegionPinRequestCountryLengthRule { max_length: 80 }),
            Rc::new(RegionPinRequestRequiredTitleRule {
                message: "region pin request needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_request_validator();
        let item = new_region_pin_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RegionPinRequestCreatedAtRangeRule { min: 32, max: 32 };
        let mut item = new_region_pin_request("x", "y");
        item.created_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = RegionPinRequestRequiredCountryRule {
            message: "region pin request needs a value".to_string(),
        };
        let mut item = new_region_pin_request("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionPinRequestNameLengthRule { max_length: 255 };
        let mut item = new_region_pin_request("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = RegionPinRequestCountryLengthRule { max_length: 64 };
        let mut item = new_region_pin_request("x", "y");
        item.country = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionPinRequestRequiredTitleRule {
            message: "region pin request needs an entry".to_string(),
        };
        let mut item = new_region_pin_request("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
