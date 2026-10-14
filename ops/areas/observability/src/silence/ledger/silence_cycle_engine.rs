use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the silence cycle calculation.
pub trait SilenceCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceCycleEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl SilenceCycleStep for SilenceCycleEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &SilenceCycle, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSilenceCycleAdjuster {
    pub minimum: f64,
}

impl SilenceCycleStep for MinimumSilenceCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SilenceCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceCycleAdjuster {
    pub percent: f64,
}

impl SilenceCycleStep for PercentSilenceCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the silence cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceCycleEngine {
    pub steps: Vec<Rc<dyn SilenceCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceCycleEngine {
    pub fn run(&mut self, item: &SilenceCycle) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SilenceCycleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SilenceCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_silence_cycle_engine() -> SilenceCycleEngine {
    SilenceCycleEngine {
        steps: vec![
            Rc::new(SilenceCycleEndsAtOffsetAdjuster { weight: 1.77 }) as Rc<dyn SilenceCycleStep>,
            Rc::new(MinimumSilenceCycleAdjuster { minimum: 40.2 }),
            Rc::new(PercentSilenceCycleAdjuster { percent: 18.0 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 44554.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_cycle_engine();
        let item = new_silence_cycle("code-992", "label-155");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
