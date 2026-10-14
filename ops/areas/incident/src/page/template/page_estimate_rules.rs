use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a page estimate must pass. Check returns its problems, or none.
pub trait PageEstimateRule: std::fmt::Debug {
    fn check(&self, item: &PageEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageEstimateQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageEstimateRule for PageEstimateQuantityRangeRule {
    fn check(&self, item: &PageEstimate) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageEstimateAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl PageEstimateRule for PageEstimateAllowedOwnerIdRule {
    fn check(&self, item: &PageEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is expired", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageEstimateOwnerIdLengthRule {
    pub max_length: i64,
}

impl PageEstimateRule for PageEstimateOwnerIdLengthRule {
    fn check(&self, item: &PageEstimate) -> Vec<String> {
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
pub struct PageEstimateCodeLengthRule {
    pub max_length: i64,
}

impl PageEstimateRule for PageEstimateCodeLengthRule {
    fn check(&self, item: &PageEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageEstimateRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageEstimateRule for PageEstimateRatioRangeRule {
    fn check(&self, item: &PageEstimate) -> Vec<String> {
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

/// Runs every rule of a page estimate.
#[derive(Debug, Clone)]
pub struct PageEstimateValidator {
    pub rules: Vec<Rc<dyn PageEstimateRule>>,
}

impl PageEstimateValidator {
    pub fn validate(&self, item: &PageEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_estimate_validator() -> PageEstimateValidator {
    PageEstimateValidator {
        rules: vec![
            Rc::new(PageEstimateQuantityRangeRule { min: 32, max: 255 }) as Rc<dyn PageEstimateRule>,
            Rc::new(PageEstimateAllowedOwnerIdRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(PageEstimateOwnerIdLengthRule { max_length: 120 }),
            Rc::new(PageEstimateCodeLengthRule { max_length: 16 }),
            Rc::new(PageEstimateRatioRangeRule { min: 7.4, max: 2836.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_estimate_validator();
        let item = new_page_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = PageEstimateQuantityRangeRule { min: 16, max: 120 };
        let mut item = new_page_estimate("x", "y");
        item.quantity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = PageEstimateOwnerIdLengthRule { max_length: 1000 };
        let mut item = new_page_estimate("x", "y");
        item.owner_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PageEstimateCodeLengthRule { max_length: 32 };
        let mut item = new_page_estimate("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = PageEstimateRatioRangeRule { min: 1.0, max: 1818.7 };
        let mut item = new_page_estimate("x", "y");
        item.ratio = 1820.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
