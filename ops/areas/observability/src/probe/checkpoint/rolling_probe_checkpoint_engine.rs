use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling probe checkpoint calculation.
pub trait RollingProbeCheckpointAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingProbeCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingProbeCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingProbeCheckpointAdjuster for TieredRollingProbeCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingProbeCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRollingProbeCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingProbeCheckpointAdjuster for NameRollingProbeCheckpointAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RollingProbeCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingProbeCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingProbeCheckpointAdjuster for FlaggedRollingProbeCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingProbeCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingProbeCheckpointAdjuster {
    pub step: f64,
}

impl RollingProbeCheckpointAdjuster for RoundingRollingProbeCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingProbeCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rolling probe checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingProbeCheckpointEngine {
    pub steps: Vec<Rc<dyn RollingProbeCheckpointAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingProbeCheckpointEngine {
    pub fn evaluate(&mut self, item: &RollingProbeCheckpoint) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[RollingProbeCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_probe_checkpoint_engine(flags: &flags::Registry) -> RollingProbeCheckpointEngine {
    RollingProbeCheckpointEngine {
        steps: vec![
            Rc::new(TieredRollingProbeCheckpointAdjuster {
                bounds: vec![2617.0, 4144.0, 8504.0],
                rates: vec![1.17, 1.05, 1.1],
            }) as Rc<dyn RollingProbeCheckpointAdjuster>,
            Rc::new(NameRollingProbeCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.31),
                    ("south".to_string(), 0.86),
                    ("online".to_string(), 0.98),
                ]),
            }),
            Rc::new(FlaggedRollingProbeCheckpointAdjuster {
                flags: flags.clone(),
                flag: "rolling-probe-checkpoint-strict".to_string(),
                percent: 0.7,
            }),
            Rc::new(RoundingRollingProbeCheckpointAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 19408.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_probe_checkpoint_engine(&flags::new_registry());
        let item = new_rolling_probe_checkpoint("code-230", "name-965");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
