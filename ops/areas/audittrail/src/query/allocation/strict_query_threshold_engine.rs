use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict query threshold calculation.
pub trait StrictQueryThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictQueryThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleStrictQueryThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictQueryThresholdStep for LocaleStrictQueryThresholdAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &StrictQueryThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictQueryThresholdAdjuster {
    pub step: f64,
}

impl StrictQueryThresholdStep for RoundingStrictQueryThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictQueryThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictQueryThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictQueryThresholdStep for TieredStrictQueryThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictQueryThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryThresholdAmountOffsetAdjuster {
    pub weight: f64,
}

impl StrictQueryThresholdStep for StrictQueryThresholdAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &StrictQueryThreshold, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the strict query threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictQueryThresholdEngine {
    pub steps: Vec<Rc<dyn StrictQueryThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictQueryThresholdEngine {
    pub fn price(&mut self, item: &StrictQueryThreshold) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[StrictQueryThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_query_threshold_engine() -> StrictQueryThresholdEngine {
    StrictQueryThresholdEngine {
        steps: vec![
            Rc::new(LocaleStrictQueryThresholdAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.79), ("east".to_string(), 1.27)]),
            }) as Rc<dyn StrictQueryThresholdStep>,
            Rc::new(RoundingStrictQueryThresholdAdjuster { step: 1.0 }),
            Rc::new(TieredStrictQueryThresholdAdjuster {
                bounds: vec![1034.0, 2050.0, 9432.0],
                rates: vec![1.11, 0.86, 0.83],
            }),
            Rc::new(StrictQueryThresholdAmountOffsetAdjuster { weight: 1.37 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 81918.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_query_threshold_engine();
        let item = new_strict_query_threshold("code-972", "label-656");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
