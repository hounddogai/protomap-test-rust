use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped probe exception calculation.
pub trait CappedProbeExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedProbeException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedProbeExceptionAdjuster {
    pub cap: f64,
}

impl CappedProbeExceptionAdjuster for CapCappedProbeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedProbeException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeExceptionLimitOffsetAdjuster {
    pub weight: f64,
}

impl CappedProbeExceptionAdjuster for CappedProbeExceptionLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &CappedProbeException, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedProbeExceptionAdjuster {
    pub minimum: f64,
}

impl CappedProbeExceptionAdjuster for MinimumCappedProbeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedProbeException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedProbeExceptionAdjuster {
    pub step: f64,
}

impl CappedProbeExceptionAdjuster for RoundingCappedProbeExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedProbeException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the capped probe exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedProbeExceptionEngine {
    pub steps: Vec<Rc<dyn CappedProbeExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedProbeExceptionEngine {
    pub fn price(&mut self, item: &CappedProbeException) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedProbeExceptionAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedProbeException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_probe_exception_engine() -> CappedProbeExceptionEngine {
    CappedProbeExceptionEngine {
        steps: vec![
            Rc::new(CapCappedProbeExceptionAdjuster { cap: 48808.0 }) as Rc<dyn CappedProbeExceptionAdjuster>,
            Rc::new(CappedProbeExceptionLimitOffsetAdjuster { weight: 1.9 }),
            Rc::new(MinimumCappedProbeExceptionAdjuster { minimum: 35.1 }),
            Rc::new(RoundingCappedProbeExceptionAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 14216.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_probe_exception_engine();
        let item = new_capped_probe_exception("code-3", "title-525");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
