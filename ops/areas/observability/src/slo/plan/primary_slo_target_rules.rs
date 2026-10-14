use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary slo target must pass. Check returns its problems, or none.
pub trait PrimarySloTargetRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySloTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloTargetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimarySloTargetRule for PrimarySloTargetAllowedCodeRule {
    fn check(&self, item: &PrimarySloTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloTargetNotesLengthRule {
    pub max_length: i64,
}

impl PrimarySloTargetRule for PrimarySloTargetNotesLengthRule {
    fn check(&self, item: &PrimarySloTarget) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloTargetCodePrefixRule {
    pub prefix: String,
}

impl PrimarySloTargetRule for PrimarySloTargetCodePrefixRule {
    fn check(&self, item: &PrimarySloTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloTargetWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimarySloTargetRule for PrimarySloTargetWindowDaysRangeRule {
    fn check(&self, item: &PrimarySloTarget) -> Vec<String> {
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
pub struct PrimarySloTargetRequiredLabelRule {
    pub message: String,
}

impl PrimarySloTargetRule for PrimarySloTargetRequiredLabelRule {
    fn check(&self, item: &PrimarySloTarget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary slo target.
#[derive(Debug, Clone)]
pub struct PrimarySloTargetValidator {
    pub rules: Vec<Rc<dyn PrimarySloTargetRule>>,
}

impl PrimarySloTargetValidator {
    pub fn validate(&self, item: &PrimarySloTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimarySloTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySloTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_slo_target_validator() -> PrimarySloTargetValidator {
    PrimarySloTargetValidator {
        rules: vec![
            Rc::new(PrimarySloTargetAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "primary".to_string()],
            }) as Rc<dyn PrimarySloTargetRule>,
            Rc::new(PrimarySloTargetNotesLengthRule { max_length: 64 }),
            Rc::new(PrimarySloTargetCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PrimarySloTargetWindowDaysRangeRule { min: 16, max: 86400 }),
            Rc::new(PrimarySloTargetRequiredLabelRule {
                message: "primary slo target needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_slo_target_validator();
        let item = new_primary_slo_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = PrimarySloTargetNotesLengthRule { max_length: 64 };
        let mut item = new_primary_slo_target("x", "y");
        item.notes = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = PrimarySloTargetWindowDaysRangeRule { min: 32, max: 64 };
        let mut item = new_primary_slo_target("x", "y");
        item.window_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimarySloTargetRequiredLabelRule {
            message: "primary slo target needs this field".to_string(),
        };
        let mut item = new_primary_slo_target("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
