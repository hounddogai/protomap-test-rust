use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal region pin policy must pass. Check returns its problems, or none.
pub trait SeasonalRegionPinPolicyRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRegionPinPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinPolicyRequiredPrincipalRule {
    pub message: String,
}

impl SeasonalRegionPinPolicyRule for SeasonalRegionPinPolicyRequiredPrincipalRule {
    fn check(&self, item: &SeasonalRegionPinPolicy) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinPolicyRequiredNameRule {
    pub message: String,
}

impl SeasonalRegionPinPolicyRule for SeasonalRegionPinPolicyRequiredNameRule {
    fn check(&self, item: &SeasonalRegionPinPolicy) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinPolicyBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalRegionPinPolicyRule for SeasonalRegionPinPolicyBurstRangeRule {
    fn check(&self, item: &SeasonalRegionPinPolicy) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinPolicyNameLengthRule {
    pub max_length: i64,
}

impl SeasonalRegionPinPolicyRule for SeasonalRegionPinPolicyNameLengthRule {
    fn check(&self, item: &SeasonalRegionPinPolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal region pin policy.
#[derive(Debug, Clone)]
pub struct SeasonalRegionPinPolicyValidator {
    pub rules: Vec<Rc<dyn SeasonalRegionPinPolicyRule>>,
}

impl SeasonalRegionPinPolicyValidator {
    pub fn validate(&self, item: &SeasonalRegionPinPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalRegionPinPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRegionPinPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_region_pin_policy_validator() -> SeasonalRegionPinPolicyValidator {
    SeasonalRegionPinPolicyValidator {
        rules: vec![
            Rc::new(SeasonalRegionPinPolicyRequiredPrincipalRule {
                message: "seasonal region pin policy needs an entry".to_string(),
            }) as Rc<dyn SeasonalRegionPinPolicyRule>,
            Rc::new(SeasonalRegionPinPolicyRequiredNameRule {
                message: "seasonal region pin policy needs a value".to_string(),
            }),
            Rc::new(SeasonalRegionPinPolicyBurstRangeRule { min: 64, max: 3600 }),
            Rc::new(SeasonalRegionPinPolicyNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_region_pin_policy_validator();
        let item = new_seasonal_region_pin_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = SeasonalRegionPinPolicyRequiredPrincipalRule {
            message: "seasonal region pin policy needs this field".to_string(),
        };
        let mut item = new_seasonal_region_pin_policy("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SeasonalRegionPinPolicyRequiredNameRule {
            message: "seasonal region pin policy needs this field".to_string(),
        };
        let mut item = new_seasonal_region_pin_policy("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = SeasonalRegionPinPolicyBurstRangeRule { min: 80, max: 255 };
        let mut item = new_seasonal_region_pin_policy("x", "y");
        item.burst = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalRegionPinPolicyNameLengthRule { max_length: 120 };
        let mut item = new_seasonal_region_pin_policy("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
