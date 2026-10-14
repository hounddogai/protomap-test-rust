use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the fallback timeline rollup calculation.
pub trait FallbackTimelineRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackTimelineRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFallbackTimelineRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackTimelineRollupFactor for NamespaceFallbackTimelineRollupAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FallbackTimelineRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackTimelineRollupAdjuster {
    pub cap: f64,
}

impl FallbackTimelineRollupFactor for CapFallbackTimelineRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackTimelineRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTimelineRollupMaximumOffsetAdjuster {
    pub weight: f64,
}

impl FallbackTimelineRollupFactor for FallbackTimelineRollupMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackTimelineRollup, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

/// Runs the fallback timeline rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackTimelineRollupEngine {
    pub steps: Vec<Rc<dyn FallbackTimelineRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackTimelineRollupEngine {
    pub fn run(&mut self, item: &FallbackTimelineRollup) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackTimelineRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackTimelineRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_timeline_rollup_engine() -> FallbackTimelineRollupEngine {
    FallbackTimelineRollupEngine {
        steps: vec![
            Rc::new(NamespaceFallbackTimelineRollupAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.23), ("north".to_string(), 0.71)]),
            }) as Rc<dyn FallbackTimelineRollupFactor>,
            Rc::new(CapFallbackTimelineRollupAdjuster { cap: 12700.0 }),
            Rc::new(FallbackTimelineRollupMaximumOffsetAdjuster { weight: 1.24 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 26131.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_timeline_rollup_engine();
        let item = new_fallback_timeline_rollup("code-623", "label-803");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
