use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the canary mapping calculation.
pub trait CanaryMappingAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryMappingAdjuster {
    pub minimum: f64,
}

impl CanaryMappingAdjuster for MinimumCanaryMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCanaryMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CanaryMappingAdjuster for FlaggedCanaryMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CanaryMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCanaryMappingAdjuster {
    pub percent: f64,
}

impl CanaryMappingAdjuster for PercentCanaryMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CanaryMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryMappingAdjuster {
    pub step: f64,
}

impl CanaryMappingAdjuster for RoundingCanaryMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the canary mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryMappingEngine {
    pub steps: Vec<Rc<dyn CanaryMappingAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryMappingEngine {
    pub fn evaluate(&mut self, item: &CanaryMapping) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[CanaryMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_canary_mapping_engine(flags: &flags::Registry) -> CanaryMappingEngine {
    CanaryMappingEngine {
        steps: vec![
            Rc::new(MinimumCanaryMappingAdjuster { minimum: 42.8 }) as Rc<dyn CanaryMappingAdjuster>,
            Rc::new(FlaggedCanaryMappingAdjuster {
                flags: flags.clone(),
                flag: "canary-mapping-pending".to_string(),
                percent: 5.6,
            }),
            Rc::new(PercentCanaryMappingAdjuster { percent: 8.2 }),
            Rc::new(RoundingCanaryMappingAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 88304.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_mapping_engine(&flags::new_registry());
        let item = new_canary_mapping("code-401", "name-492");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
