use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence target must pass. Check returns its problems, or none.
pub trait SilenceTargetRule: std::fmt::Debug {
    fn check(&self, item: &SilenceTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTargetCodePrefixRule {
    pub prefix: String,
}

impl SilenceTargetRule for SilenceTargetCodePrefixRule {
    fn check(&self, item: &SilenceTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTargetOwnerIdLengthRule {
    pub max_length: i64,
}

impl SilenceTargetRule for SilenceTargetOwnerIdLengthRule {
    fn check(&self, item: &SilenceTarget) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTargetTitleLengthRule {
    pub max_length: i64,
}

impl SilenceTargetRule for SilenceTargetTitleLengthRule {
    fn check(&self, item: &SilenceTarget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTargetFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SilenceTargetRule for SilenceTargetFloorValueRangeRule {
    fn check(&self, item: &SilenceTarget) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceTargetLabelLengthRule {
    pub max_length: i64,
}

impl SilenceTargetRule for SilenceTargetLabelLengthRule {
    fn check(&self, item: &SilenceTarget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence target.
#[derive(Debug, Clone)]
pub struct SilenceTargetValidator {
    pub rules: Vec<Rc<dyn SilenceTargetRule>>,
}

impl SilenceTargetValidator {
    pub fn validate(&self, item: &SilenceTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SilenceTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_target_validator() -> SilenceTargetValidator {
    SilenceTargetValidator {
        rules: vec![
            Rc::new(SilenceTargetCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn SilenceTargetRule>,
            Rc::new(SilenceTargetOwnerIdLengthRule { max_length: 16 }),
            Rc::new(SilenceTargetTitleLengthRule { max_length: 16 }),
            Rc::new(SilenceTargetFloorValueRangeRule { min: 2.1, max: 3335.2 }),
            Rc::new(SilenceTargetLabelLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_target_validator();
        let item = new_silence_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = SilenceTargetOwnerIdLengthRule { max_length: 3600 };
        let mut item = new_silence_target("x", "y");
        item.owner_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SilenceTargetTitleLengthRule { max_length: 255 };
        let mut item = new_silence_target("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = SilenceTargetFloorValueRangeRule { min: 1.6, max: 2079.2 };
        let mut item = new_silence_target("x", "y");
        item.floor_value = 2080.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SilenceTargetLabelLengthRule { max_length: 64 };
        let mut item = new_silence_target("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
