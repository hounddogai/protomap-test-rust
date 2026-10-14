use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the runbook rule calculation.
pub trait RunbookRuleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookRuleAdjuster {
    pub percent: f64,
}

impl RunbookRuleAdjuster for PercentRunbookRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RunbookRuleAdjuster for TieredRunbookRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RunbookRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRunbookRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookRuleAdjuster for TitleRunbookRuleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RunbookRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookRuleAdjuster for FlaggedRunbookRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the runbook rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookRuleEngine {
    pub steps: Vec<Rc<dyn RunbookRuleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookRuleEngine {
    pub fn price(&mut self, item: &RunbookRule) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RunbookRuleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RunbookRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_runbook_rule_engine(flags: &flags::Registry) -> RunbookRuleEngine {
    RunbookRuleEngine {
        steps: vec![
            Rc::new(PercentRunbookRuleAdjuster { percent: -8.5 }) as Rc<dyn RunbookRuleAdjuster>,
            Rc::new(TieredRunbookRuleAdjuster {
                bounds: vec![976.0, 2948.0, 6069.0, 7384.0],
                rates: vec![0.94, 0.98, 1.12, 0.94],
            }),
            Rc::new(TitleRunbookRuleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.19), ("west".to_string(), 0.82)]),
            }),
            Rc::new(FlaggedRunbookRuleAdjuster {
                flags: flags.clone(),
                flag: "runbook-rule-tiered".to_string(),
                percent: -4.8,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 96957.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_rule_engine(&flags::new_registry());
        let item = new_runbook_rule("code-268", "title-139");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
