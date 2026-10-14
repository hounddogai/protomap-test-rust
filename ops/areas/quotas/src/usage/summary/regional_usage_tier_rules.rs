use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional usage tier must pass. Check returns its problems, or none.
pub trait RegionalUsageTierRule: std::fmt::Debug {
    fn check(&self, item: &RegionalUsageTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageTierRequiredNameRule {
    pub message: String,
}

impl RegionalUsageTierRule for RegionalUsageTierRequiredNameRule {
    fn check(&self, item: &RegionalUsageTier) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageTierRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalUsageTierRule for RegionalUsageTierRatioRangeRule {
    fn check(&self, item: &RegionalUsageTier) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageTierNameLengthRule {
    pub max_length: i64,
}

impl RegionalUsageTierRule for RegionalUsageTierNameLengthRule {
    fn check(&self, item: &RegionalUsageTier) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional usage tier.
#[derive(Debug, Clone)]
pub struct RegionalUsageTierValidator {
    pub rules: Vec<Rc<dyn RegionalUsageTierRule>>,
}

impl RegionalUsageTierValidator {
    pub fn validate(&self, item: &RegionalUsageTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalUsageTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalUsageTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_usage_tier_validator() -> RegionalUsageTierValidator {
    RegionalUsageTierValidator {
        rules: vec![
            Rc::new(RegionalUsageTierRequiredNameRule {
                message: "regional usage tier needs an entry".to_string(),
            }) as Rc<dyn RegionalUsageTierRule>,
            Rc::new(RegionalUsageTierRatioRangeRule { min: 2.8, max: 2098.6 }),
            Rc::new(RegionalUsageTierNameLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_usage_tier_validator();
        let item = new_regional_usage_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalUsageTierRequiredNameRule {
            message: "regional usage tier needs this field".to_string(),
        };
        let mut item = new_regional_usage_tier("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = RegionalUsageTierRatioRangeRule { min: 7.7, max: 131.7 };
        let mut item = new_regional_usage_tier("x", "y");
        item.ratio = 133.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionalUsageTierNameLengthRule { max_length: 16 };
        let mut item = new_regional_usage_tier("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
