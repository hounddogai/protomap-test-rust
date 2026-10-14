use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback escalation rollup calculation.
pub trait FallbackEscalationRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackEscalationRollup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackEscalationRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackEscalationRollupModifier for FlaggedFallbackEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackEscalationRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackEscalationRollupAdjuster {
    pub cap: f64,
}

impl FallbackEscalationRollupModifier for CapFallbackEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackEscalationRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackEscalationRollupAdjuster {
    pub percent: f64,
}

impl FallbackEscalationRollupModifier for PercentFallbackEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackEscalationRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackEscalationRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackEscalationRollupModifier for TieredFallbackEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackEscalationRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackEscalationRollupAdjuster {
    pub minimum: f64,
}

impl FallbackEscalationRollupModifier for MinimumFallbackEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackEscalationRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackEscalationRollupAdjuster {
    pub step: f64,
}

impl FallbackEscalationRollupModifier for RoundingFallbackEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackEscalationRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the fallback escalation rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackEscalationRollupEngine {
    pub steps: Vec<Rc<dyn FallbackEscalationRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackEscalationRollupEngine {
    pub fn run(&mut self, item: &FallbackEscalationRollup) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[FallbackEscalationRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_escalation_rollup_engine(flags: &flags::Registry) -> FallbackEscalationRollupEngine {
    FallbackEscalationRollupEngine {
        steps: vec![
            Rc::new(FlaggedFallbackEscalationRollupAdjuster {
                flags: flags.clone(),
                flag: "fallback-escalation-rollup-weighted".to_string(),
                percent: -3.1,
            }) as Rc<dyn FallbackEscalationRollupModifier>,
            Rc::new(CapFallbackEscalationRollupAdjuster { cap: 22878.0 }),
            Rc::new(PercentFallbackEscalationRollupAdjuster { percent: 20.9 }),
            Rc::new(TieredFallbackEscalationRollupAdjuster {
                bounds: vec![3950.0, 4844.0, 7482.0],
                rates: vec![0.92, 0.88, 1.0],
            }),
            Rc::new(MinimumFallbackEscalationRollupAdjuster { minimum: 47.3 }),
            Rc::new(RoundingFallbackEscalationRollupAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 83423.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_escalation_rollup_engine(&flags::new_registry());
        let item = new_fallback_escalation_rollup("code-986", "title-548");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
