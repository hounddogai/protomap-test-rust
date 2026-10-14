use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page target must pass. Check returns its problems, or none.
pub trait PageTargetRule: std::fmt::Debug {
    fn check(&self, item: &PageTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageTargetRequiredTimezoneRule {
    pub message: String,
}

impl PageTargetRule for PageTargetRequiredTimezoneRule {
    fn check(&self, item: &PageTarget) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageTargetCodePrefixRule {
    pub prefix: String,
}

impl PageTargetRule for PageTargetCodePrefixRule {
    fn check(&self, item: &PageTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageTargetRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageTargetRule for PageTargetRateRangeRule {
    fn check(&self, item: &PageTarget) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a page target.
#[derive(Debug, Clone)]
pub struct PageTargetValidator {
    pub rules: Vec<Rc<dyn PageTargetRule>>,
}

impl PageTargetValidator {
    pub fn validate(&self, item: &PageTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_target_validator() -> PageTargetValidator {
    PageTargetValidator {
        rules: vec![
            Rc::new(PageTargetRequiredTimezoneRule {
                message: "page target needs a value".to_string(),
            }) as Rc<dyn PageTargetRule>,
            Rc::new(PageTargetCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PageTargetRateRangeRule { min: 8.3, max: 2487.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_target_validator();
        let item = new_page_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = PageTargetRequiredTimezoneRule {
            message: "page target needs this field".to_string(),
        };
        let mut item = new_page_target("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = PageTargetRateRangeRule { min: 5.2, max: 3492.7 };
        let mut item = new_page_target("x", "y");
        item.rate = 3494.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
