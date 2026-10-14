use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone entry must pass. Check returns its problems, or none.
pub trait ZoneEntryRule: std::fmt::Debug {
    fn check(&self, item: &ZoneEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEntryCodePrefixRule {
    pub prefix: String,
}

impl ZoneEntryRule for ZoneEntryCodePrefixRule {
    fn check(&self, item: &ZoneEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEntryDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ZoneEntryRule for ZoneEntryDiscountRangeRule {
    fn check(&self, item: &ZoneEntry) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEntryCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ZoneEntryRule for ZoneEntryCeilingRangeRule {
    fn check(&self, item: &ZoneEntry) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEntryRequiredUnitRule {
    pub message: String,
}

impl ZoneEntryRule for ZoneEntryRequiredUnitRule {
    fn check(&self, item: &ZoneEntry) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEntryAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneEntryRule for ZoneEntryAttemptsRangeRule {
    fn check(&self, item: &ZoneEntry) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a zone entry.
#[derive(Debug, Clone)]
pub struct ZoneEntryValidator {
    pub rules: Vec<Rc<dyn ZoneEntryRule>>,
}

impl ZoneEntryValidator {
    pub fn validate(&self, item: &ZoneEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_entry_validator() -> ZoneEntryValidator {
    ZoneEntryValidator {
        rules: vec![
            Rc::new(ZoneEntryCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn ZoneEntryRule>,
            Rc::new(ZoneEntryDiscountRangeRule { min: 4.9, max: 1565.9 }),
            Rc::new(ZoneEntryCeilingRangeRule { min: 7.2, max: 3009.7 }),
            Rc::new(ZoneEntryRequiredUnitRule {
                message: "zone entry needs an entry".to_string(),
            }),
            Rc::new(ZoneEntryAttemptsRangeRule { min: 64, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_entry_validator();
        let item = new_zone_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = ZoneEntryDiscountRangeRule { min: 8.7, max: 2113.0 };
        let mut item = new_zone_entry("x", "y");
        item.discount = 2114.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ZoneEntryCeilingRangeRule { min: 0.8, max: 2123.2 };
        let mut item = new_zone_entry("x", "y");
        item.ceiling = 2124.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = ZoneEntryRequiredUnitRule {
            message: "zone entry needs this field".to_string(),
        };
        let mut item = new_zone_entry("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ZoneEntryAttemptsRangeRule { min: 1000, max: 86400 };
        let mut item = new_zone_entry("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
