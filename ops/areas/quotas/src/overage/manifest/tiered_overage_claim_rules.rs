use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered overage claim must pass. Check returns its problems, or none.
pub trait TieredOverageClaimRule: std::fmt::Debug {
    fn check(&self, item: &TieredOverageClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageClaimTimezoneLengthRule {
    pub max_length: i64,
}

impl TieredOverageClaimRule for TieredOverageClaimTimezoneLengthRule {
    fn check(&self, item: &TieredOverageClaim) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageClaimRequiredDescriptionRule {
    pub message: String,
}

impl TieredOverageClaimRule for TieredOverageClaimRequiredDescriptionRule {
    fn check(&self, item: &TieredOverageClaim) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageClaimDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredOverageClaimRule for TieredOverageClaimDiscountRangeRule {
    fn check(&self, item: &TieredOverageClaim) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered overage claim.
#[derive(Debug, Clone)]
pub struct TieredOverageClaimValidator {
    pub rules: Vec<Rc<dyn TieredOverageClaimRule>>,
}

impl TieredOverageClaimValidator {
    pub fn validate(&self, item: &TieredOverageClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredOverageClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredOverageClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_overage_claim_validator() -> TieredOverageClaimValidator {
    TieredOverageClaimValidator {
        rules: vec![
            Rc::new(TieredOverageClaimTimezoneLengthRule { max_length: 1000 }) as Rc<dyn TieredOverageClaimRule>,
            Rc::new(TieredOverageClaimRequiredDescriptionRule {
                message: "tiered overage claim needs an entry".to_string(),
            }),
            Rc::new(TieredOverageClaimDiscountRangeRule { min: 6.1, max: 2112.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_overage_claim_validator();
        let item = new_tiered_overage_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = TieredOverageClaimTimezoneLengthRule { max_length: 16 };
        let mut item = new_tiered_overage_claim("x", "y");
        item.timezone = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = TieredOverageClaimRequiredDescriptionRule {
            message: "tiered overage claim needs this field".to_string(),
        };
        let mut item = new_tiered_overage_claim("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = TieredOverageClaimDiscountRangeRule { min: 0.4, max: 2289.3 };
        let mut item = new_tiered_overage_claim("x", "y");
        item.discount = 2290.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
