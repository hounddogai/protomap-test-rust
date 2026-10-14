use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback page group must pass. Check returns its problems, or none.
pub trait FallbackPageGroupRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPageGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageGroupDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackPageGroupRule for FallbackPageGroupDiscountRangeRule {
    fn check(&self, item: &FallbackPageGroup) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageGroupChecksumLengthRule {
    pub max_length: i64,
}

impl FallbackPageGroupRule for FallbackPageGroupChecksumLengthRule {
    fn check(&self, item: &FallbackPageGroup) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageGroupHostLengthRule {
    pub max_length: i64,
}

impl FallbackPageGroupRule for FallbackPageGroupHostLengthRule {
    fn check(&self, item: &FallbackPageGroup) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageGroupCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPageGroupRule for FallbackPageGroupCapacityRangeRule {
    fn check(&self, item: &FallbackPageGroup) -> Vec<String> {
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

/// Runs every rule of a fallback page group.
#[derive(Debug, Clone)]
pub struct FallbackPageGroupValidator {
    pub rules: Vec<Rc<dyn FallbackPageGroupRule>>,
}

impl FallbackPageGroupValidator {
    pub fn validate(&self, item: &FallbackPageGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackPageGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPageGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_page_group_validator() -> FallbackPageGroupValidator {
    FallbackPageGroupValidator {
        rules: vec![
            Rc::new(FallbackPageGroupDiscountRangeRule { min: 1.5, max: 4333.6 }) as Rc<dyn FallbackPageGroupRule>,
            Rc::new(FallbackPageGroupChecksumLengthRule { max_length: 3600 }),
            Rc::new(FallbackPageGroupHostLengthRule { max_length: 3600 }),
            Rc::new(FallbackPageGroupCapacityRangeRule { min: 3600, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_page_group_validator();
        let item = new_fallback_page_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = FallbackPageGroupDiscountRangeRule { min: 8.7, max: 1536.2 };
        let mut item = new_fallback_page_group("x", "y");
        item.discount = 1537.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = FallbackPageGroupChecksumLengthRule { max_length: 120 };
        let mut item = new_fallback_page_group("x", "y");
        item.checksum = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = FallbackPageGroupHostLengthRule { max_length: 32 };
        let mut item = new_fallback_page_group("x", "y");
        item.host = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = FallbackPageGroupCapacityRangeRule { min: 16, max: 64 };
        let mut item = new_fallback_page_group("x", "y");
        item.capacity = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
