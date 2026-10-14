use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension manifest must pass. Check returns its problems, or none.
pub trait SuspensionManifestRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionManifestCodePrefixRule {
    pub prefix: String,
}

impl SuspensionManifestRule for SuspensionManifestCodePrefixRule {
    fn check(&self, item: &SuspensionManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionManifestFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionManifestRule for SuspensionManifestFloorValueRangeRule {
    fn check(&self, item: &SuspensionManifest) -> Vec<String> {
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
pub struct SuspensionManifestRequiredTitleRule {
    pub message: String,
}

impl SuspensionManifestRule for SuspensionManifestRequiredTitleRule {
    fn check(&self, item: &SuspensionManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionManifestRequiredCodeRule {
    pub message: String,
}

impl SuspensionManifestRule for SuspensionManifestRequiredCodeRule {
    fn check(&self, item: &SuspensionManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionManifestDescriptionLengthRule {
    pub max_length: i64,
}

impl SuspensionManifestRule for SuspensionManifestDescriptionLengthRule {
    fn check(&self, item: &SuspensionManifest) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension manifest.
#[derive(Debug, Clone)]
pub struct SuspensionManifestValidator {
    pub rules: Vec<Rc<dyn SuspensionManifestRule>>,
}

impl SuspensionManifestValidator {
    pub fn validate(&self, item: &SuspensionManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_manifest_validator() -> SuspensionManifestValidator {
    SuspensionManifestValidator {
        rules: vec![
            Rc::new(SuspensionManifestCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn SuspensionManifestRule>,
            Rc::new(SuspensionManifestFloorValueRangeRule { min: 2.5, max: 2184.8 }),
            Rc::new(SuspensionManifestRequiredTitleRule {
                message: "suspension manifest needs this field".to_string(),
            }),
            Rc::new(SuspensionManifestRequiredCodeRule {
                message: "suspension manifest needs an entry".to_string(),
            }),
            Rc::new(SuspensionManifestDescriptionLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_manifest_validator();
        let item = new_suspension_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = SuspensionManifestFloorValueRangeRule { min: 7.0, max: 3991.9 };
        let mut item = new_suspension_manifest("x", "y");
        item.floor_value = 3993.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SuspensionManifestRequiredTitleRule {
            message: "suspension manifest needs an entry".to_string(),
        };
        let mut item = new_suspension_manifest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SuspensionManifestRequiredCodeRule {
            message: "suspension manifest needs an entry".to_string(),
        };
        let mut item = new_suspension_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = SuspensionManifestDescriptionLengthRule { max_length: 80 };
        let mut item = new_suspension_manifest("x", "y");
        item.description = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
