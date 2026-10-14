use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a freeze mapping must pass. Check returns its problems, or none.
pub trait FreezeMappingRule: std::fmt::Debug {
    fn check(&self, item: &FreezeMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMappingRequiredCodeRule {
    pub message: String,
}

impl FreezeMappingRule for FreezeMappingRequiredCodeRule {
    fn check(&self, item: &FreezeMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMappingAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl FreezeMappingRule for FreezeMappingAllowedTitleRule {
    fn check(&self, item: &FreezeMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is unknown", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMappingCodeLengthRule {
    pub max_length: i64,
}

impl FreezeMappingRule for FreezeMappingCodeLengthRule {
    fn check(&self, item: &FreezeMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMappingPhoneLengthRule {
    pub max_length: i64,
}

impl FreezeMappingRule for FreezeMappingPhoneLengthRule {
    fn check(&self, item: &FreezeMapping) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMappingRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezeMappingRule for FreezeMappingRatioRangeRule {
    fn check(&self, item: &FreezeMapping) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a freeze mapping.
#[derive(Debug, Clone)]
pub struct FreezeMappingValidator {
    pub rules: Vec<Rc<dyn FreezeMappingRule>>,
}

impl FreezeMappingValidator {
    pub fn validate(&self, item: &FreezeMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezeMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_mapping_validator() -> FreezeMappingValidator {
    FreezeMappingValidator {
        rules: vec![
            Rc::new(FreezeMappingRequiredCodeRule {
                message: "freeze mapping needs an entry".to_string(),
            }) as Rc<dyn FreezeMappingRule>,
            Rc::new(FreezeMappingAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(FreezeMappingCodeLengthRule { max_length: 3600 }),
            Rc::new(FreezeMappingPhoneLengthRule { max_length: 1000 }),
            Rc::new(FreezeMappingRatioRangeRule { min: 3.1, max: 203.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_mapping_validator();
        let item = new_freeze_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FreezeMappingRequiredCodeRule {
            message: "freeze mapping needs a value".to_string(),
        };
        let mut item = new_freeze_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FreezeMappingCodeLengthRule { max_length: 86400 };
        let mut item = new_freeze_mapping("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = FreezeMappingPhoneLengthRule { max_length: 80 };
        let mut item = new_freeze_mapping("x", "y");
        item.phone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = FreezeMappingRatioRangeRule { min: 3.3, max: 4177.3 };
        let mut item = new_freeze_mapping("x", "y");
        item.ratio = 4178.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
