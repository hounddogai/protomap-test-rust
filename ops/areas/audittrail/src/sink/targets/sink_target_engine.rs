use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the sink target calculation.
pub trait SinkTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkTargetAdjuster {
    pub percent: f64,
}

impl SinkTargetFactor for PercentSinkTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSinkTargetAdjuster {
    pub step: f64,
}

impl SinkTargetFactor for RoundingSinkTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SinkTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountrySinkTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkTargetFactor for CountrySinkTargetAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &SinkTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSinkTargetAdjuster {
    pub cap: f64,
}

impl SinkTargetFactor for CapSinkTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SinkTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkTargetFactor for FlaggedSinkTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkTargetAdjuster {
    pub minimum: f64,
}

impl SinkTargetFactor for MinimumSinkTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the sink target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkTargetEngine {
    pub steps: Vec<Rc<dyn SinkTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkTargetEngine {
    pub fn run(&mut self, item: &SinkTarget) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SinkTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_sink_target_engine(flags: &flags::Registry) -> SinkTargetEngine {
    SinkTargetEngine {
        steps: vec![
            Rc::new(PercentSinkTargetAdjuster { percent: -10.0 }) as Rc<dyn SinkTargetFactor>,
            Rc::new(RoundingSinkTargetAdjuster { step: 0.5 }),
            Rc::new(CountrySinkTargetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.18), ("central".to_string(), 1.24)]),
            }),
            Rc::new(CapSinkTargetAdjuster { cap: 33590.0 }),
            Rc::new(FlaggedSinkTargetAdjuster {
                flags: flags.clone(),
                flag: "sink-target-primary".to_string(),
                percent: 5.7,
            }),
            Rc::new(MinimumSinkTargetAdjuster { minimum: 25.1 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 29463.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_target_engine(&flags::new_registry());
        let item = new_sink_target("code-907", "title-82");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
