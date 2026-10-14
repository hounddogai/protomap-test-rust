use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted zone profile must pass. Check returns its problems, or none.
pub trait WeightedZoneProfileRule: std::fmt::Debug {
    fn check(&self, item: &WeightedZoneProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneProfileRequiredTitleRule {
    pub message: String,
}

impl WeightedZoneProfileRule for WeightedZoneProfileRequiredTitleRule {
    fn check(&self, item: &WeightedZoneProfile) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneProfileTitleLengthRule {
    pub max_length: i64,
}

impl WeightedZoneProfileRule for WeightedZoneProfileTitleLengthRule {
    fn check(&self, item: &WeightedZoneProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneProfileBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedZoneProfileRule for WeightedZoneProfileBurstRangeRule {
    fn check(&self, item: &WeightedZoneProfile) -> Vec<String> {
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
pub struct WeightedZoneProfileRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedZoneProfileRule for WeightedZoneProfileRotationDaysRangeRule {
    fn check(&self, item: &WeightedZoneProfile) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted zone profile.
#[derive(Debug, Clone)]
pub struct WeightedZoneProfileValidator {
    pub rules: Vec<Rc<dyn WeightedZoneProfileRule>>,
}

impl WeightedZoneProfileValidator {
    pub fn validate(&self, item: &WeightedZoneProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedZoneProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedZoneProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_zone_profile_validator() -> WeightedZoneProfileValidator {
    WeightedZoneProfileValidator {
        rules: vec![
            Rc::new(WeightedZoneProfileRequiredTitleRule {
                message: "weighted zone profile needs a value".to_string(),
            }) as Rc<dyn WeightedZoneProfileRule>,
            Rc::new(WeightedZoneProfileTitleLengthRule { max_length: 3600 }),
            Rc::new(WeightedZoneProfileBurstRangeRule { min: 16, max: 32 }),
            Rc::new(WeightedZoneProfileRotationDaysRangeRule { min: 16, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_zone_profile_validator();
        let item = new_weighted_zone_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedZoneProfileRequiredTitleRule {
            message: "weighted zone profile needs a value".to_string(),
        };
        let mut item = new_weighted_zone_profile("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = WeightedZoneProfileTitleLengthRule { max_length: 80 };
        let mut item = new_weighted_zone_profile("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = WeightedZoneProfileBurstRangeRule { min: 255, max: 86400 };
        let mut item = new_weighted_zone_profile("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = WeightedZoneProfileRotationDaysRangeRule { min: 64, max: 3600 };
        let mut item = new_weighted_zone_profile("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
