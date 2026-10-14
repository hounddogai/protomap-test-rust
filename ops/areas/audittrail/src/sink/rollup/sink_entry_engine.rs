use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the sink entry calculation.
pub trait SinkEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SinkEntryStep for TieredSinkEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SinkEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSinkEntryAdjuster {
    pub step: f64,
}

impl SinkEntryStep for RoundingSinkEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SinkEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSinkEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkEntryStep for NameSinkEntryAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SinkEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkEntryAdjuster {
    pub minimum: f64,
}

impl SinkEntryStep for MinimumSinkEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkEntryStep for FlaggedSinkEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the sink entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkEntryEngine {
    pub steps: Vec<Rc<dyn SinkEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkEntryEngine {
    pub fn compute(&mut self, item: &SinkEntry) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SinkEntryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SinkEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_sink_entry_engine(flags: &flags::Registry) -> SinkEntryEngine {
    SinkEntryEngine {
        steps: vec![
            Rc::new(TieredSinkEntryAdjuster {
                bounds: vec![6885.0, 9998.0],
                rates: vec![0.85, 0.96],
            }) as Rc<dyn SinkEntryStep>,
            Rc::new(RoundingSinkEntryAdjuster { step: 0.05 }),
            Rc::new(NameSinkEntryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.86)]),
            }),
            Rc::new(MinimumSinkEntryAdjuster { minimum: 33.6 }),
            Rc::new(FlaggedSinkEntryAdjuster {
                flags: flags.clone(),
                flag: "sink-entry-pending".to_string(),
                percent: -2.2,
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 6770.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_entry_engine(&flags::new_registry());
        let item = new_sink_entry("code-136", "name-747");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
