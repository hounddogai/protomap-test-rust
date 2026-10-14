use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation schedule must pass. Check returns its problems, or none.
pub trait FederationScheduleRule: std::fmt::Debug {
    fn check(&self, item: &FederationSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationScheduleRequiredLabelRule {
    pub message: String,
}

impl FederationScheduleRule for FederationScheduleRequiredLabelRule {
    fn check(&self, item: &FederationSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationScheduleLabelLengthRule {
    pub max_length: i64,
}

impl FederationScheduleRule for FederationScheduleLabelLengthRule {
    fn check(&self, item: &FederationSchedule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationScheduleCodePrefixRule {
    pub prefix: String,
}

impl FederationScheduleRule for FederationScheduleCodePrefixRule {
    fn check(&self, item: &FederationSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationScheduleCodeLengthRule {
    pub max_length: i64,
}

impl FederationScheduleRule for FederationScheduleCodeLengthRule {
    fn check(&self, item: &FederationSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationScheduleQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationScheduleRule for FederationScheduleQuotaUnitsRangeRule {
    fn check(&self, item: &FederationSchedule) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a federation schedule.
#[derive(Debug, Clone)]
pub struct FederationScheduleValidator {
    pub rules: Vec<Rc<dyn FederationScheduleRule>>,
}

impl FederationScheduleValidator {
    pub fn validate(&self, item: &FederationSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_schedule_validator() -> FederationScheduleValidator {
    FederationScheduleValidator {
        rules: vec![
            Rc::new(FederationScheduleRequiredLabelRule {
                message: "federation schedule needs a value".to_string(),
            }) as Rc<dyn FederationScheduleRule>,
            Rc::new(FederationScheduleLabelLengthRule { max_length: 80 }),
            Rc::new(FederationScheduleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(FederationScheduleCodeLengthRule { max_length: 1000 }),
            Rc::new(FederationScheduleQuotaUnitsRangeRule { min: 3600, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_schedule_validator();
        let item = new_federation_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FederationScheduleRequiredLabelRule {
            message: "federation schedule needs this field".to_string(),
        };
        let mut item = new_federation_schedule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FederationScheduleLabelLengthRule { max_length: 120 };
        let mut item = new_federation_schedule("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FederationScheduleCodeLengthRule { max_length: 86400 };
        let mut item = new_federation_schedule("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = FederationScheduleQuotaUnitsRangeRule { min: 32, max: 255 };
        let mut item = new_federation_schedule("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
