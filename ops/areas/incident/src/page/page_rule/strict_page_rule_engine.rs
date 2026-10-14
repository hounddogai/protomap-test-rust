use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict page rule calculation.
pub trait StrictPageRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictPageRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictPageRuleAdjuster {
    pub step: f64,
}

impl StrictPageRuleModifier for RoundingStrictPageRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictPageRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictPageRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictPageRuleModifier for FlaggedStrictPageRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictPageRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageRuleBudgetOffsetAdjuster {
    pub weight: f64,
}

impl StrictPageRuleModifier for StrictPageRuleBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &StrictPageRule, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictPageRuleAdjuster {
    pub cap: f64,
}

impl StrictPageRuleModifier for CapStrictPageRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictPageRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the strict page rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictPageRuleEngine {
    pub steps: Vec<Rc<dyn StrictPageRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictPageRuleEngine {
    pub fn run(&mut self, item: &StrictPageRule) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictPageRuleModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[StrictPageRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_page_rule_engine(flags: &flags::Registry) -> StrictPageRuleEngine {
    StrictPageRuleEngine {
        steps: vec![
            Rc::new(RoundingStrictPageRuleAdjuster { step: 5.0 }) as Rc<dyn StrictPageRuleModifier>,
            Rc::new(FlaggedStrictPageRuleAdjuster {
                flags: flags.clone(),
                flag: "strict-page-rule-primary".to_string(),
                percent: 1.0,
            }),
            Rc::new(StrictPageRuleBudgetOffsetAdjuster { weight: 0.38 }),
            Rc::new(CapStrictPageRuleAdjuster { cap: 27677.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 38242.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_page_rule_engine(&flags::new_registry());
        let item = new_strict_page_rule("code-958", "title-635");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
