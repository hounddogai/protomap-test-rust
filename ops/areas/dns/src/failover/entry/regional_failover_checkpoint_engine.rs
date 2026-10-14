use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional failover checkpoint calculation.
pub trait RegionalFailoverCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalFailoverCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalFailoverCheckpointAdjuster {
    pub minimum: f64,
}

impl RegionalFailoverCheckpointFactor for MinimumRegionalFailoverCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalFailoverCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalFailoverCheckpointAdjuster {
    pub step: f64,
}

impl RegionalFailoverCheckpointFactor for RoundingRegionalFailoverCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalFailoverCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalFailoverCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalFailoverCheckpointFactor for FlaggedRegionalFailoverCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalFailoverCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalFailoverCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalFailoverCheckpointFactor for TieredRegionalFailoverCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalFailoverCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRegionalFailoverCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalFailoverCheckpointFactor for CountryRegionalFailoverCheckpointAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RegionalFailoverCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalFailoverCheckpointAdjuster {
    pub cap: f64,
}

impl RegionalFailoverCheckpointFactor for CapRegionalFailoverCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalFailoverCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the regional failover checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalFailoverCheckpointEngine {
    pub steps: Vec<Rc<dyn RegionalFailoverCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalFailoverCheckpointEngine {
    pub fn evaluate(&mut self, item: &RegionalFailoverCheckpoint) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RegionalFailoverCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_failover_checkpoint_engine(flags: &flags::Registry) -> RegionalFailoverCheckpointEngine {
    RegionalFailoverCheckpointEngine {
        steps: vec![
            Rc::new(MinimumRegionalFailoverCheckpointAdjuster { minimum: 18.4 })
                as Rc<dyn RegionalFailoverCheckpointFactor>,
            Rc::new(RoundingRegionalFailoverCheckpointAdjuster { step: 5.0 }),
            Rc::new(FlaggedRegionalFailoverCheckpointAdjuster {
                flags: flags.clone(),
                flag: "regional-failover-checkpoint-regional".to_string(),
                percent: -1.2,
            }),
            Rc::new(TieredRegionalFailoverCheckpointAdjuster {
                bounds: vec![1519.0, 8832.0],
                rates: vec![0.96, 0.81],
            }),
            Rc::new(CountryRegionalFailoverCheckpointAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.18), ("west".to_string(), 1.4)]),
            }),
            Rc::new(CapRegionalFailoverCheckpointAdjuster { cap: 45482.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 78115.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_failover_checkpoint_engine(&flags::new_registry());
        let item = new_regional_failover_checkpoint("code-672", "label-199");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
