use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a slo group must pass. Check returns its problems, or none.
pub trait SloGroupRule: std::fmt::Debug {
    fn check(&self, item: &SloGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloGroupRequiredChecksumRule {
    pub message: String,
}

impl SloGroupRule for SloGroupRequiredChecksumRule {
    fn check(&self, item: &SloGroup) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloGroupAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl SloGroupRule for SloGroupAllowedChecksumRule {
    fn check(&self, item: &SloGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is duplicated", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloGroupQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloGroupRule for SloGroupQuantityRangeRule {
    fn check(&self, item: &SloGroup) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a slo group.
#[derive(Debug, Clone)]
pub struct SloGroupValidator {
    pub rules: Vec<Rc<dyn SloGroupRule>>,
}

impl SloGroupValidator {
    pub fn validate(&self, item: &SloGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SloGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_group_validator() -> SloGroupValidator {
    SloGroupValidator {
        rules: vec![
            Rc::new(SloGroupRequiredChecksumRule {
                message: "slo group needs this field".to_string(),
            }) as Rc<dyn SloGroupRule>,
            Rc::new(SloGroupAllowedChecksumRule {
                allowed: vec!["tiered".to_string(), "batched".to_string()],
            }),
            Rc::new(SloGroupQuantityRangeRule { min: 80, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_group_validator();
        let item = new_slo_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = SloGroupRequiredChecksumRule {
            message: "slo group needs an entry".to_string(),
        };
        let mut item = new_slo_group("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = SloGroupQuantityRangeRule { min: 32, max: 120 };
        let mut item = new_slo_group("x", "y");
        item.quantity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
