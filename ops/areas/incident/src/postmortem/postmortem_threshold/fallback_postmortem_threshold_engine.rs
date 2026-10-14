use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback postmortem threshold calculation.
pub trait FallbackPostmortemThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPostmortemThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackPostmortemThresholdAdjuster {
    pub minimum: f64,
}

impl FallbackPostmortemThresholdStep for MinimumFallbackPostmortemThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackPostmortemThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackPostmortemThresholdAdjuster {
    pub step: f64,
}

impl FallbackPostmortemThresholdStep for RoundingFallbackPostmortemThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackPostmortemThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackPostmortemThresholdAdjuster {
    pub cap: f64,
}

impl FallbackPostmortemThresholdStep for CapFallbackPostmortemThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackPostmortemThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemThresholdAmountOffsetAdjuster {
    pub weight: f64,
}

impl FallbackPostmortemThresholdStep for FallbackPostmortemThresholdAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackPostmortemThreshold, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackPostmortemThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackPostmortemThresholdStep for TieredFallbackPostmortemThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackPostmortemThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the fallback postmortem threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPostmortemThresholdEngine {
    pub steps: Vec<Rc<dyn FallbackPostmortemThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPostmortemThresholdEngine {
    pub fn compute(&mut self, item: &FallbackPostmortemThreshold) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackPostmortemThresholdStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackPostmortemThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_postmortem_threshold_engine() -> FallbackPostmortemThresholdEngine {
    FallbackPostmortemThresholdEngine {
        steps: vec![
            Rc::new(MinimumFallbackPostmortemThresholdAdjuster { minimum: 30.4 })
                as Rc<dyn FallbackPostmortemThresholdStep>,
            Rc::new(RoundingFallbackPostmortemThresholdAdjuster { step: 0.05 }),
            Rc::new(CapFallbackPostmortemThresholdAdjuster { cap: 37713.0 }),
            Rc::new(FallbackPostmortemThresholdAmountOffsetAdjuster { weight: 1.35 }),
            Rc::new(TieredFallbackPostmortemThresholdAdjuster {
                bounds: vec![1102.0, 1754.0, 4124.0, 5815.0],
                rates: vec![1.02, 1.06, 1.2, 0.88],
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 70786.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_postmortem_threshold_engine();
        let item = new_fallback_postmortem_threshold("code-842", "title-816");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
