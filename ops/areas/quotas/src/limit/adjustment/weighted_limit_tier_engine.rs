use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted limit tier calculation.
pub trait WeightedLimitTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedLimitTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedLimitTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedLimitTierStep for TieredWeightedLimitTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedLimitTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedLimitTierAdjuster {
    pub minimum: f64,
}

impl WeightedLimitTierStep for MinimumWeightedLimitTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedLimitTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedLimitTierAdjuster {
    pub percent: f64,
}

impl WeightedLimitTierStep for PercentWeightedLimitTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedLimitTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedLimitTierAdjuster {
    pub cap: f64,
}

impl WeightedLimitTierStep for CapWeightedLimitTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedLimitTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedLimitTierAdjuster {
    pub step: f64,
}

impl WeightedLimitTierStep for RoundingWeightedLimitTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedLimitTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitTierRatioOffsetAdjuster {
    pub weight: f64,
}

impl WeightedLimitTierStep for WeightedLimitTierRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedLimitTier, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

/// Runs the weighted limit tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedLimitTierEngine {
    pub steps: Vec<Rc<dyn WeightedLimitTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedLimitTierEngine {
    pub fn price(&mut self, item: &WeightedLimitTier) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[WeightedLimitTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_limit_tier_engine() -> WeightedLimitTierEngine {
    WeightedLimitTierEngine {
        steps: vec![
            Rc::new(TieredWeightedLimitTierAdjuster {
                bounds: vec![596.0, 2133.0, 2349.0],
                rates: vec![0.83, 0.87, 1.19],
            }) as Rc<dyn WeightedLimitTierStep>,
            Rc::new(MinimumWeightedLimitTierAdjuster { minimum: 29.8 }),
            Rc::new(PercentWeightedLimitTierAdjuster { percent: -1.2 }),
            Rc::new(CapWeightedLimitTierAdjuster { cap: 8201.0 }),
            Rc::new(RoundingWeightedLimitTierAdjuster { step: 5.0 }),
            Rc::new(WeightedLimitTierRatioOffsetAdjuster { weight: 0.24 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 78874.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_limit_tier_engine();
        let item = new_weighted_limit_tier("code-921", "name-264");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
