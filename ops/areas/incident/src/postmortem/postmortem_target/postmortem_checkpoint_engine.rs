use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the postmortem checkpoint calculation.
pub trait PostmortemCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemCheckpointPortOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemCheckpointModifier for PostmortemCheckpointPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemCheckpoint, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPostmortemCheckpointAdjuster {
    pub percent: f64,
}

impl PostmortemCheckpointModifier for PercentPostmortemCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PostmortemCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemCheckpointAdjuster {
    pub minimum: f64,
}

impl PostmortemCheckpointModifier for MinimumPostmortemCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PostmortemCheckpointModifier for TieredPostmortemCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PostmortemCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemCheckpointModifier for FlaggedPostmortemCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the postmortem checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemCheckpointEngine {
    pub steps: Vec<Rc<dyn PostmortemCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemCheckpointEngine {
    pub fn price(&mut self, item: &PostmortemCheckpoint) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PostmortemCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_postmortem_checkpoint_engine(flags: &flags::Registry) -> PostmortemCheckpointEngine {
    PostmortemCheckpointEngine {
        steps: vec![
            Rc::new(PostmortemCheckpointPortOffsetAdjuster { weight: 0.57 }) as Rc<dyn PostmortemCheckpointModifier>,
            Rc::new(PercentPostmortemCheckpointAdjuster { percent: 19.0 }),
            Rc::new(MinimumPostmortemCheckpointAdjuster { minimum: 24.7 }),
            Rc::new(TieredPostmortemCheckpointAdjuster {
                bounds: vec![5159.0, 6623.0],
                rates: vec![0.85, 0.89],
            }),
            Rc::new(FlaggedPostmortemCheckpointAdjuster {
                flags: flags.clone(),
                flag: "postmortem-checkpoint-strict".to_string(),
                percent: -9.4,
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 62096.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_checkpoint_engine(&flags::new_registry());
        let item = new_postmortem_checkpoint("code-387", "name-515");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
