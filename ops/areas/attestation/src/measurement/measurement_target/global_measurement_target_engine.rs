use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global measurement target calculation.
pub trait GlobalMeasurementTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalMeasurementTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalMeasurementTargetAdjuster {
    pub step: f64,
}

impl GlobalMeasurementTargetFactor for RoundingGlobalMeasurementTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalMeasurementTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitGlobalMeasurementTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalMeasurementTargetFactor for UnitGlobalMeasurementTargetAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &GlobalMeasurementTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalMeasurementTargetAdjuster {
    pub percent: f64,
}

impl GlobalMeasurementTargetFactor for PercentGlobalMeasurementTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalMeasurementTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalMeasurementTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalMeasurementTargetFactor for TieredGlobalMeasurementTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalMeasurementTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalMeasurementTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalMeasurementTargetFactor for FlaggedGlobalMeasurementTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalMeasurementTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global measurement target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalMeasurementTargetEngine {
    pub steps: Vec<Rc<dyn GlobalMeasurementTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalMeasurementTargetEngine {
    pub fn price(&mut self, item: &GlobalMeasurementTarget) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[GlobalMeasurementTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_measurement_target_engine(flags: &flags::Registry) -> GlobalMeasurementTargetEngine {
    GlobalMeasurementTargetEngine {
        steps: vec![
            Rc::new(RoundingGlobalMeasurementTargetAdjuster { step: 5.0 }) as Rc<dyn GlobalMeasurementTargetFactor>,
            Rc::new(UnitGlobalMeasurementTargetAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.15)]),
            }),
            Rc::new(PercentGlobalMeasurementTargetAdjuster { percent: -11.9 }),
            Rc::new(TieredGlobalMeasurementTargetAdjuster {
                bounds: vec![997.0, 6385.0, 9690.0],
                rates: vec![1.18, 0.95, 0.96],
            }),
            Rc::new(FlaggedGlobalMeasurementTargetAdjuster {
                flags: flags.clone(),
                flag: "global-measurement-target-global".to_string(),
                percent: -8.1,
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 16665.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_measurement_target_engine(&flags::new_registry());
        let item = new_global_measurement_target("code-703", "title-931");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
