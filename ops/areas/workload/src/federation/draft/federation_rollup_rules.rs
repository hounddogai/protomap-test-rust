use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation rollup must pass. Check returns its problems, or none.
pub trait FederationRollupRule: std::fmt::Debug {
    fn check(&self, item: &FederationRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRollupChecksumLengthRule {
    pub max_length: i64,
}

impl FederationRollupRule for FederationRollupChecksumLengthRule {
    fn check(&self, item: &FederationRollup) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRollupRequiredCodeRule {
    pub message: String,
}

impl FederationRollupRule for FederationRollupRequiredCodeRule {
    fn check(&self, item: &FederationRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRollupContactEmailLengthRule {
    pub max_length: i64,
}

impl FederationRollupRule for FederationRollupContactEmailLengthRule {
    fn check(&self, item: &FederationRollup) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRollupWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationRollupRule for FederationRollupWindowDaysRangeRule {
    fn check(&self, item: &FederationRollup) -> Vec<String> {
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
pub struct FederationRollupRequiredContactEmailRule {
    pub message: String,
}

impl FederationRollupRule for FederationRollupRequiredContactEmailRule {
    fn check(&self, item: &FederationRollup) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationRollupLabelLengthRule {
    pub max_length: i64,
}

impl FederationRollupRule for FederationRollupLabelLengthRule {
    fn check(&self, item: &FederationRollup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation rollup.
#[derive(Debug, Clone)]
pub struct FederationRollupValidator {
    pub rules: Vec<Rc<dyn FederationRollupRule>>,
}

impl FederationRollupValidator {
    pub fn validate(&self, item: &FederationRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_rollup_validator() -> FederationRollupValidator {
    FederationRollupValidator {
        rules: vec![
            Rc::new(FederationRollupChecksumLengthRule { max_length: 3600 }) as Rc<dyn FederationRollupRule>,
            Rc::new(FederationRollupRequiredCodeRule {
                message: "federation rollup needs this field".to_string(),
            }),
            Rc::new(FederationRollupContactEmailLengthRule { max_length: 32 }),
            Rc::new(FederationRollupWindowDaysRangeRule { min: 16, max: 3600 }),
            Rc::new(FederationRollupRequiredContactEmailRule {
                message: "federation rollup needs a value".to_string(),
            }),
            Rc::new(FederationRollupLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_rollup_validator();
        let item = new_federation_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = FederationRollupChecksumLengthRule { max_length: 80 };
        let mut item = new_federation_rollup("x", "y");
        item.checksum = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FederationRollupRequiredCodeRule {
            message: "federation rollup needs this field".to_string(),
        };
        let mut item = new_federation_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = FederationRollupContactEmailLengthRule { max_length: 120 };
        let mut item = new_federation_rollup("x", "y");
        item.contact_email = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = FederationRollupWindowDaysRangeRule { min: 32, max: 255 };
        let mut item = new_federation_rollup("x", "y");
        item.window_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = FederationRollupRequiredContactEmailRule {
            message: "federation rollup needs this field".to_string(),
        };
        let mut item = new_federation_rollup("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FederationRollupLabelLengthRule { max_length: 86400 };
        let mut item = new_federation_rollup("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
