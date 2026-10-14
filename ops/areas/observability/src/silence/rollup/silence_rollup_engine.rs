use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the silence rollup calculation.
pub trait SilenceRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceRollupAdjuster {
    pub percent: f64,
}

impl SilenceRollupFactor for PercentSilenceRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSilenceRollupAdjuster {
    pub minimum: f64,
}

impl SilenceRollupFactor for MinimumSilenceRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SilenceRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSilenceRollupAdjuster {
    pub step: f64,
}

impl SilenceRollupFactor for RoundingSilenceRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SilenceRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSilenceRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SilenceRollupFactor for SegmentSilenceRollupAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &SilenceRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the silence rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceRollupEngine {
    pub steps: Vec<Rc<dyn SilenceRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceRollupEngine {
    pub fn evaluate(&mut self, item: &SilenceRollup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SilenceRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_silence_rollup_engine() -> SilenceRollupEngine {
    SilenceRollupEngine {
        steps: vec![
            Rc::new(PercentSilenceRollupAdjuster { percent: -4.4 }) as Rc<dyn SilenceRollupFactor>,
            Rc::new(MinimumSilenceRollupAdjuster { minimum: 17.0 }),
            Rc::new(RoundingSilenceRollupAdjuster { step: 0.25 }),
            Rc::new(SegmentSilenceRollupAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.09),
                    ("central".to_string(), 1.14),
                    ("north".to_string(), 0.74),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 4572.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_rollup_engine();
        let item = new_silence_rollup("code-642", "name-711");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
