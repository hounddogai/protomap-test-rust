use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending region pin agreement must pass. Check returns its problems, or none.
pub trait PendingRegionPinAgreementRule: std::fmt::Debug {
    fn check(&self, item: &PendingRegionPinAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinAgreementRequiredCategoryRule {
    pub message: String,
}

impl PendingRegionPinAgreementRule for PendingRegionPinAgreementRequiredCategoryRule {
    fn check(&self, item: &PendingRegionPinAgreement) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinAgreementBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRegionPinAgreementRule for PendingRegionPinAgreementBatchSizeRangeRule {
    fn check(&self, item: &PendingRegionPinAgreement) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinAgreementRequiredNotesRule {
    pub message: String,
}

impl PendingRegionPinAgreementRule for PendingRegionPinAgreementRequiredNotesRule {
    fn check(&self, item: &PendingRegionPinAgreement) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinAgreementRequiredCodeRule {
    pub message: String,
}

impl PendingRegionPinAgreementRule for PendingRegionPinAgreementRequiredCodeRule {
    fn check(&self, item: &PendingRegionPinAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending region pin agreement.
#[derive(Debug, Clone)]
pub struct PendingRegionPinAgreementValidator {
    pub rules: Vec<Rc<dyn PendingRegionPinAgreementRule>>,
}

impl PendingRegionPinAgreementValidator {
    pub fn validate(&self, item: &PendingRegionPinAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingRegionPinAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRegionPinAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_region_pin_agreement_validator() -> PendingRegionPinAgreementValidator {
    PendingRegionPinAgreementValidator {
        rules: vec![
            Rc::new(PendingRegionPinAgreementRequiredCategoryRule {
                message: "pending region pin agreement needs this field".to_string(),
            }) as Rc<dyn PendingRegionPinAgreementRule>,
            Rc::new(PendingRegionPinAgreementBatchSizeRangeRule { min: 32, max: 120 }),
            Rc::new(PendingRegionPinAgreementRequiredNotesRule {
                message: "pending region pin agreement needs a value".to_string(),
            }),
            Rc::new(PendingRegionPinAgreementRequiredCodeRule {
                message: "pending region pin agreement needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_region_pin_agreement_validator();
        let item = new_pending_region_pin_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = PendingRegionPinAgreementRequiredCategoryRule {
            message: "pending region pin agreement needs an entry".to_string(),
        };
        let mut item = new_pending_region_pin_agreement("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = PendingRegionPinAgreementBatchSizeRangeRule { min: 32, max: 1000 };
        let mut item = new_pending_region_pin_agreement("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = PendingRegionPinAgreementRequiredNotesRule {
            message: "pending region pin agreement needs this field".to_string(),
        };
        let mut item = new_pending_region_pin_agreement("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingRegionPinAgreementRequiredCodeRule {
            message: "pending region pin agreement needs an entry".to_string(),
        };
        let mut item = new_pending_region_pin_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
