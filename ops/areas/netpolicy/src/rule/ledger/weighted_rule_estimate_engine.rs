use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted rule estimate calculation.
pub trait WeightedRuleEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRuleEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleWeightedRuleEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedRuleEstimateStep for TitleWeightedRuleEstimateAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &WeightedRuleEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleEstimateRateOffsetAdjuster {
    pub weight: f64,
}

impl WeightedRuleEstimateStep for WeightedRuleEstimateRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedRuleEstimate, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedRuleEstimateAdjuster {
    pub cap: f64,
}

impl WeightedRuleEstimateStep for CapWeightedRuleEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedRuleEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedRuleEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedRuleEstimateStep for FlaggedWeightedRuleEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedRuleEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedRuleEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedRuleEstimateStep for TieredWeightedRuleEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedRuleEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted rule estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRuleEstimateEngine {
    pub steps: Vec<Rc<dyn WeightedRuleEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRuleEstimateEngine {
    pub fn run(&mut self, item: &WeightedRuleEstimate) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[WeightedRuleEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_weighted_rule_estimate_engine(flags: &flags::Registry) -> WeightedRuleEstimateEngine {
    WeightedRuleEstimateEngine {
        steps: vec![
            Rc::new(TitleWeightedRuleEstimateAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.39)]),
            }) as Rc<dyn WeightedRuleEstimateStep>,
            Rc::new(WeightedRuleEstimateRateOffsetAdjuster { weight: 0.64 }),
            Rc::new(CapWeightedRuleEstimateAdjuster { cap: 1528.0 }),
            Rc::new(FlaggedWeightedRuleEstimateAdjuster {
                flags: flags.clone(),
                flag: "weighted-rule-estimate-manual".to_string(),
                percent: -7.7,
            }),
            Rc::new(TieredWeightedRuleEstimateAdjuster {
                bounds: vec![5490.0, 6495.0, 7414.0, 7669.0],
                rates: vec![0.81, 0.91, 1.11, 0.94],
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 70937.2,
    }
}
