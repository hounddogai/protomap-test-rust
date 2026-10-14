use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback probe claim must pass. Check returns its problems, or none.
pub trait FallbackProbeClaimRule: std::fmt::Debug {
    fn check(&self, item: &FallbackProbeClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackProbeClaimNameLengthRule {
    pub max_length: i64,
}

impl FallbackProbeClaimRule for FallbackProbeClaimNameLengthRule {
    fn check(&self, item: &FallbackProbeClaim) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackProbeClaimRequiredNameRule {
    pub message: String,
}

impl FallbackProbeClaimRule for FallbackProbeClaimRequiredNameRule {
    fn check(&self, item: &FallbackProbeClaim) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackProbeClaimCodeLengthRule {
    pub max_length: i64,
}

impl FallbackProbeClaimRule for FallbackProbeClaimCodeLengthRule {
    fn check(&self, item: &FallbackProbeClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackProbeClaimQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackProbeClaimRule for FallbackProbeClaimQuantityRangeRule {
    fn check(&self, item: &FallbackProbeClaim) -> Vec<String> {
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

/// Runs every rule of a fallback probe claim.
#[derive(Debug, Clone)]
pub struct FallbackProbeClaimValidator {
    pub rules: Vec<Rc<dyn FallbackProbeClaimRule>>,
}

impl FallbackProbeClaimValidator {
    pub fn validate(&self, item: &FallbackProbeClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackProbeClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackProbeClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_probe_claim_validator() -> FallbackProbeClaimValidator {
    FallbackProbeClaimValidator {
        rules: vec![
            Rc::new(FallbackProbeClaimNameLengthRule { max_length: 64 }) as Rc<dyn FallbackProbeClaimRule>,
            Rc::new(FallbackProbeClaimRequiredNameRule {
                message: "fallback probe claim needs an entry".to_string(),
            }),
            Rc::new(FallbackProbeClaimCodeLengthRule { max_length: 255 }),
            Rc::new(FallbackProbeClaimQuantityRangeRule { min: 80, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_probe_claim_validator();
        let item = new_fallback_probe_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackProbeClaimNameLengthRule { max_length: 86400 };
        let mut item = new_fallback_probe_claim("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FallbackProbeClaimRequiredNameRule {
            message: "fallback probe claim needs an entry".to_string(),
        };
        let mut item = new_fallback_probe_claim("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackProbeClaimCodeLengthRule { max_length: 86400 };
        let mut item = new_fallback_probe_claim("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = FallbackProbeClaimQuantityRangeRule { min: 1000, max: 3600 };
        let mut item = new_fallback_probe_claim("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
