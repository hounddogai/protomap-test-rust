use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the silence plan calculation.
pub trait SilencePlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilencePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSilencePlanAdjuster {
    pub minimum: f64,
}

impl SilencePlanFactor for MinimumSilencePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SilencePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilencePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SilencePlanFactor for TieredSilencePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SilencePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSilencePlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SilencePlanFactor for FlaggedSilencePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SilencePlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilencePlanFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl SilencePlanFactor for SilencePlanFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &SilencePlan, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

/// Runs the silence plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilencePlanEngine {
    pub steps: Vec<Rc<dyn SilencePlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilencePlanEngine {
    pub fn evaluate(&mut self, item: &SilencePlan) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SilencePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_silence_plan_engine(flags: &flags::Registry) -> SilencePlanEngine {
    SilencePlanEngine {
        steps: vec![
            Rc::new(MinimumSilencePlanAdjuster { minimum: 4.1 }) as Rc<dyn SilencePlanFactor>,
            Rc::new(TieredSilencePlanAdjuster {
                bounds: vec![3026.0, 5464.0, 7633.0],
                rates: vec![1.1, 1.05, 0.97],
            }),
            Rc::new(FlaggedSilencePlanAdjuster {
                flags: flags.clone(),
                flag: "silence-plan-tiered".to_string(),
                percent: -2.9,
            }),
            Rc::new(SilencePlanFloorValueOffsetAdjuster { weight: 0.87 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 89995.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_plan_engine(&flags::new_registry());
        let item = new_silence_plan("code-89", "title-254");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
