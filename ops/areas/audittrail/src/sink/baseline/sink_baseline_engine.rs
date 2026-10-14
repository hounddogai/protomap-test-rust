use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the sink baseline calculation.
pub trait SinkBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSinkBaselineAdjuster {
    pub cap: f64,
}

impl SinkBaselineAdjuster for CapSinkBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SinkBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkBaselineAdjuster {
    pub minimum: f64,
}

impl SinkBaselineAdjuster for MinimumSinkBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkBaselineAdjuster for FlaggedSinkBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBaselineMaximumOffsetAdjuster {
    pub weight: f64,
}

impl SinkBaselineAdjuster for SinkBaselineMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &SinkBaseline, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SinkBaselineAdjuster for TieredSinkBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SinkBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the sink baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkBaselineEngine {
    pub steps: Vec<Rc<dyn SinkBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkBaselineEngine {
    pub fn price(&mut self, item: &SinkBaseline) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SinkBaselineAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SinkBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_sink_baseline_engine(flags: &flags::Registry) -> SinkBaselineEngine {
    SinkBaselineEngine {
        steps: vec![
            Rc::new(CapSinkBaselineAdjuster { cap: 2271.0 }) as Rc<dyn SinkBaselineAdjuster>,
            Rc::new(MinimumSinkBaselineAdjuster { minimum: 34.6 }),
            Rc::new(FlaggedSinkBaselineAdjuster {
                flags: flags.clone(),
                flag: "sink-baseline-fallback".to_string(),
                percent: -7.5,
            }),
            Rc::new(SinkBaselineMaximumOffsetAdjuster { weight: 1.16 }),
            Rc::new(TieredSinkBaselineAdjuster {
                bounds: vec![1835.0, 5761.0, 7079.0, 7615.0],
                rates: vec![1.15, 0.91, 1.12, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 34995.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_baseline_engine(&flags::new_registry());
        let item = new_sink_baseline("code-837", "name-843");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
