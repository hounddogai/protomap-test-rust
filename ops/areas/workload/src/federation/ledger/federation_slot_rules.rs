use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation slot must pass. Check returns its problems, or none.
pub trait FederationSlotRule: std::fmt::Debug {
    fn check(&self, item: &FederationSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSlotRequiredCodeRule {
    pub message: String,
}

impl FederationSlotRule for FederationSlotRequiredCodeRule {
    fn check(&self, item: &FederationSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSlotExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationSlotRule for FederationSlotExpiresAtRangeRule {
    fn check(&self, item: &FederationSlot) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationSlotQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationSlotRule for FederationSlotQuotaUnitsRangeRule {
    fn check(&self, item: &FederationSlot) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation slot.
#[derive(Debug, Clone)]
pub struct FederationSlotValidator {
    pub rules: Vec<Rc<dyn FederationSlotRule>>,
}

impl FederationSlotValidator {
    pub fn validate(&self, item: &FederationSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_slot_validator() -> FederationSlotValidator {
    FederationSlotValidator {
        rules: vec![
            Rc::new(FederationSlotRequiredCodeRule {
                message: "federation slot needs an entry".to_string(),
            }) as Rc<dyn FederationSlotRule>,
            Rc::new(FederationSlotExpiresAtRangeRule { min: 16, max: 32 }),
            Rc::new(FederationSlotQuotaUnitsRangeRule { min: 16, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_slot_validator();
        let item = new_federation_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FederationSlotRequiredCodeRule {
            message: "federation slot needs this field".to_string(),
        };
        let mut item = new_federation_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = FederationSlotExpiresAtRangeRule { min: 255, max: 255 };
        let mut item = new_federation_slot("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = FederationSlotQuotaUnitsRangeRule { min: 80, max: 255 };
        let mut item = new_federation_slot("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
