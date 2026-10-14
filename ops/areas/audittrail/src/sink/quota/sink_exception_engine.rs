use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the sink exception calculation.
pub trait SinkExceptionFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkExceptionAdjuster {
    pub percent: f64,
}

impl SinkExceptionFactor for PercentSinkExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkExceptionAdjuster {
    pub minimum: f64,
}

impl SinkExceptionFactor for MinimumSinkExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceSinkExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkExceptionFactor for NamespaceSinkExceptionAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &SinkException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkExceptionFactor for FlaggedSinkExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SinkExceptionFactor for TieredSinkExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SinkException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkExceptionScoreOffsetAdjuster {
    pub weight: f64,
}

impl SinkExceptionFactor for SinkExceptionScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &SinkException, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the sink exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkExceptionEngine {
    pub steps: Vec<Rc<dyn SinkExceptionFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkExceptionEngine {
    pub fn compute(&mut self, item: &SinkException) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SinkExceptionFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SinkException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_sink_exception_engine(flags: &flags::Registry) -> SinkExceptionEngine {
    SinkExceptionEngine {
        steps: vec![
            Rc::new(PercentSinkExceptionAdjuster { percent: 12.4 }) as Rc<dyn SinkExceptionFactor>,
            Rc::new(MinimumSinkExceptionAdjuster { minimum: 46.1 }),
            Rc::new(NamespaceSinkExceptionAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.34), ("south".to_string(), 1.03)]),
            }),
            Rc::new(FlaggedSinkExceptionAdjuster {
                flags: flags.clone(),
                flag: "sink-exception-strict".to_string(),
                percent: 3.6,
            }),
            Rc::new(TieredSinkExceptionAdjuster {
                bounds: vec![2016.0, 4535.0, 9911.0],
                rates: vec![0.89, 0.84, 1.19],
            }),
            Rc::new(SinkExceptionScoreOffsetAdjuster { weight: 1.34 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 30069.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_exception_engine(&flags::new_registry());
        let item = new_sink_exception("code-25", "name-942");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
