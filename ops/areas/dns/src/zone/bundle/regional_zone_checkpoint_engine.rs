use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional zone checkpoint calculation.
pub trait RegionalZoneCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalZoneCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalZoneCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalZoneCheckpointModifier for FlaggedRegionalZoneCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalZoneCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRegionalZoneCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalZoneCheckpointModifier for LocaleRegionalZoneCheckpointAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RegionalZoneCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneCheckpointRetriesOffsetAdjuster {
    pub weight: f64,
}

impl RegionalZoneCheckpointModifier for RegionalZoneCheckpointRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalZoneCheckpoint, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalZoneCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalZoneCheckpointModifier for TieredRegionalZoneCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalZoneCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalZoneCheckpointAdjuster {
    pub cap: f64,
}

impl RegionalZoneCheckpointModifier for CapRegionalZoneCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalZoneCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the regional zone checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalZoneCheckpointEngine {
    pub steps: Vec<Rc<dyn RegionalZoneCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalZoneCheckpointEngine {
    pub fn evaluate(&mut self, item: &RegionalZoneCheckpoint) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[RegionalZoneCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_zone_checkpoint_engine(flags: &flags::Registry) -> RegionalZoneCheckpointEngine {
    RegionalZoneCheckpointEngine {
        steps: vec![
            Rc::new(FlaggedRegionalZoneCheckpointAdjuster {
                flags: flags.clone(),
                flag: "regional-zone-checkpoint-adaptive".to_string(),
                percent: -0.5,
            }) as Rc<dyn RegionalZoneCheckpointModifier>,
            Rc::new(LocaleRegionalZoneCheckpointAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.28)]),
            }),
            Rc::new(RegionalZoneCheckpointRetriesOffsetAdjuster { weight: 1.67 }),
            Rc::new(TieredRegionalZoneCheckpointAdjuster {
                bounds: vec![5579.0, 6706.0],
                rates: vec![1.09, 1.04],
            }),
            Rc::new(CapRegionalZoneCheckpointAdjuster { cap: 6502.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 47635.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_zone_checkpoint_engine(&flags::new_registry());
        let item = new_regional_zone_checkpoint("code-368", "label-296");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
