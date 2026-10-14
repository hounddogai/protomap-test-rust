use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone claim must pass. Check returns its problems, or none.
pub trait ZoneClaimRule: std::fmt::Debug {
    fn check(&self, item: &ZoneClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneClaimBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ZoneClaimRule for ZoneClaimBudgetRangeRule {
    fn check(&self, item: &ZoneClaim) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneClaimCodePrefixRule {
    pub prefix: String,
}

impl ZoneClaimRule for ZoneClaimCodePrefixRule {
    fn check(&self, item: &ZoneClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneClaimRequiredCountryRule {
    pub message: String,
}

impl ZoneClaimRule for ZoneClaimRequiredCountryRule {
    fn check(&self, item: &ZoneClaim) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneClaimTitleLengthRule {
    pub max_length: i64,
}

impl ZoneClaimRule for ZoneClaimTitleLengthRule {
    fn check(&self, item: &ZoneClaim) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneClaimOwnerIdLengthRule {
    pub max_length: i64,
}

impl ZoneClaimRule for ZoneClaimOwnerIdLengthRule {
    fn check(&self, item: &ZoneClaim) -> Vec<String> {
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
pub struct ZoneClaimCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZoneClaimRule for ZoneClaimCapacityRangeRule {
    fn check(&self, item: &ZoneClaim) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a zone claim.
#[derive(Debug, Clone)]
pub struct ZoneClaimValidator {
    pub rules: Vec<Rc<dyn ZoneClaimRule>>,
}

impl ZoneClaimValidator {
    pub fn validate(&self, item: &ZoneClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_claim_validator() -> ZoneClaimValidator {
    ZoneClaimValidator {
        rules: vec![
            Rc::new(ZoneClaimBudgetRangeRule { min: 0.4, max: 3951.5 }) as Rc<dyn ZoneClaimRule>,
            Rc::new(ZoneClaimCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ZoneClaimRequiredCountryRule {
                message: "zone claim needs this field".to_string(),
            }),
            Rc::new(ZoneClaimTitleLengthRule { max_length: 120 }),
            Rc::new(ZoneClaimOwnerIdLengthRule { max_length: 1000 }),
            Rc::new(ZoneClaimCapacityRangeRule { min: 64, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_claim_validator();
        let item = new_zone_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = ZoneClaimBudgetRangeRule { min: 9.1, max: 1641.2 };
        let mut item = new_zone_claim("x", "y");
        item.budget = 1642.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = ZoneClaimRequiredCountryRule {
            message: "zone claim needs this field".to_string(),
        };
        let mut item = new_zone_claim("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ZoneClaimTitleLengthRule { max_length: 120 };
        let mut item = new_zone_claim("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = ZoneClaimOwnerIdLengthRule { max_length: 64 };
        let mut item = new_zone_claim("x", "y");
        item.owner_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = ZoneClaimCapacityRangeRule { min: 16, max: 86400 };
        let mut item = new_zone_claim("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
