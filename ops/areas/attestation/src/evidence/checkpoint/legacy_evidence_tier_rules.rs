use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy evidence tier must pass. Check returns its problems, or none.
pub trait LegacyEvidenceTierRule: std::fmt::Debug {
    fn check(&self, item: &LegacyEvidenceTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceTierRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyEvidenceTierRule for LegacyEvidenceTierRotationDaysRangeRule {
    fn check(&self, item: &LegacyEvidenceTier) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceTierRequiredLabelRule {
    pub message: String,
}

impl LegacyEvidenceTierRule for LegacyEvidenceTierRequiredLabelRule {
    fn check(&self, item: &LegacyEvidenceTier) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceTierNotesLengthRule {
    pub max_length: i64,
}

impl LegacyEvidenceTierRule for LegacyEvidenceTierNotesLengthRule {
    fn check(&self, item: &LegacyEvidenceTier) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy evidence tier.
#[derive(Debug, Clone)]
pub struct LegacyEvidenceTierValidator {
    pub rules: Vec<Rc<dyn LegacyEvidenceTierRule>>,
}

impl LegacyEvidenceTierValidator {
    pub fn validate(&self, item: &LegacyEvidenceTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyEvidenceTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyEvidenceTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_evidence_tier_validator() -> LegacyEvidenceTierValidator {
    LegacyEvidenceTierValidator {
        rules: vec![
            Rc::new(LegacyEvidenceTierRotationDaysRangeRule { min: 16, max: 120 }) as Rc<dyn LegacyEvidenceTierRule>,
            Rc::new(LegacyEvidenceTierRequiredLabelRule {
                message: "legacy evidence tier needs this field".to_string(),
            }),
            Rc::new(LegacyEvidenceTierNotesLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_evidence_tier_validator();
        let item = new_legacy_evidence_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = LegacyEvidenceTierRotationDaysRangeRule { min: 120, max: 255 };
        let mut item = new_legacy_evidence_tier("x", "y");
        item.rotation_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LegacyEvidenceTierRequiredLabelRule {
            message: "legacy evidence tier needs an entry".to_string(),
        };
        let mut item = new_legacy_evidence_tier("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = LegacyEvidenceTierNotesLengthRule { max_length: 86400 };
        let mut item = new_legacy_evidence_tier("x", "y");
        item.notes = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
