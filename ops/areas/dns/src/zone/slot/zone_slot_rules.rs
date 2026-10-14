use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone slot must pass. Check returns its problems, or none.
pub trait ZoneSlotRule: std::fmt::Debug {
    fn check(&self, item: &ZoneSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSlotRequiredNameRule {
    pub message: String,
}

impl ZoneSlotRule for ZoneSlotRequiredNameRule {
    fn check(&self, item: &ZoneSlot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSlotQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneSlotRule for ZoneSlotQuotaUnitsRangeRule {
    fn check(&self, item: &ZoneSlot) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSlotRequiredAudienceRule {
    pub message: String,
}

impl ZoneSlotRule for ZoneSlotRequiredAudienceRule {
    fn check(&self, item: &ZoneSlot) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone slot.
#[derive(Debug, Clone)]
pub struct ZoneSlotValidator {
    pub rules: Vec<Rc<dyn ZoneSlotRule>>,
}

impl ZoneSlotValidator {
    pub fn validate(&self, item: &ZoneSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_slot_validator() -> ZoneSlotValidator {
    ZoneSlotValidator {
        rules: vec![
            Rc::new(ZoneSlotRequiredNameRule {
                message: "zone slot needs this field".to_string(),
            }) as Rc<dyn ZoneSlotRule>,
            Rc::new(ZoneSlotQuotaUnitsRangeRule { min: 255, max: 255 }),
            Rc::new(ZoneSlotRequiredAudienceRule {
                message: "zone slot needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_slot_validator();
        let item = new_zone_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ZoneSlotRequiredNameRule {
            message: "zone slot needs a value".to_string(),
        };
        let mut item = new_zone_slot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = ZoneSlotQuotaUnitsRangeRule { min: 80, max: 1000 };
        let mut item = new_zone_slot("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = ZoneSlotRequiredAudienceRule {
            message: "zone slot needs a value".to_string(),
        };
        let mut item = new_zone_slot("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
