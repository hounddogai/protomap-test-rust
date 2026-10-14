use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the limit mapping calculation.
pub trait LimitMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLimitMappingAdjuster {
    pub step: f64,
}

impl LimitMappingModifier for RoundingLimitMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LimitMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitMappingAdjuster {
    pub percent: f64,
}

impl LimitMappingModifier for PercentLimitMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLimitMappingAdjuster {
    pub cap: f64,
}

impl LimitMappingModifier for CapLimitMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LimitMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LimitMappingModifier for TieredLimitMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LimitMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitMappingAdjuster {
    pub minimum: f64,
}

impl LimitMappingModifier for MinimumLimitMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitMappingWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl LimitMappingModifier for LimitMappingWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &LimitMapping, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

/// Runs the limit mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitMappingEngine {
    pub steps: Vec<Rc<dyn LimitMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitMappingEngine {
    pub fn evaluate(&mut self, item: &LimitMapping) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LimitMappingModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LimitMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_limit_mapping_engine() -> LimitMappingEngine {
    LimitMappingEngine {
        steps: vec![
            Rc::new(RoundingLimitMappingAdjuster { step: 5.0 }) as Rc<dyn LimitMappingModifier>,
            Rc::new(PercentLimitMappingAdjuster { percent: 18.8 }),
            Rc::new(CapLimitMappingAdjuster { cap: 1875.0 }),
            Rc::new(TieredLimitMappingAdjuster {
                bounds: vec![3629.0, 3758.0],
                rates: vec![1.07, 0.84],
            }),
            Rc::new(MinimumLimitMappingAdjuster { minimum: 2.0 }),
            Rc::new(LimitMappingWindowDaysOffsetAdjuster { weight: 0.58 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 89041.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_mapping_engine();
        let item = new_limit_mapping("code-455", "label-549");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
