use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the capped postmortem cycle calculation.
pub trait CappedPostmortemCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedPostmortemCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedPostmortemCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedPostmortemCycleStep for TieredCappedPostmortemCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedPostmortemCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedPostmortemCycleAdjuster {
    pub cap: f64,
}

impl CappedPostmortemCycleStep for CapCappedPostmortemCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedPostmortemCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedPostmortemCycleAdjuster {
    pub minimum: f64,
}

impl CappedPostmortemCycleStep for MinimumCappedPostmortemCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedPostmortemCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemCycleDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl CappedPostmortemCycleStep for CappedPostmortemCycleDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &CappedPostmortemCycle, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceCappedPostmortemCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedPostmortemCycleStep for SourceCappedPostmortemCycleAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &CappedPostmortemCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the capped postmortem cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedPostmortemCycleEngine {
    pub steps: Vec<Rc<dyn CappedPostmortemCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedPostmortemCycleEngine {
    pub fn price(&mut self, item: &CappedPostmortemCycle) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedPostmortemCycleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedPostmortemCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_postmortem_cycle_engine() -> CappedPostmortemCycleEngine {
    CappedPostmortemCycleEngine {
        steps: vec![
            Rc::new(TieredCappedPostmortemCycleAdjuster {
                bounds: vec![2365.0, 4130.0, 4887.0, 9002.0],
                rates: vec![1.17, 1.15, 0.85, 0.95],
            }) as Rc<dyn CappedPostmortemCycleStep>,
            Rc::new(CapCappedPostmortemCycleAdjuster { cap: 19275.0 }),
            Rc::new(MinimumCappedPostmortemCycleAdjuster { minimum: 4.6 }),
            Rc::new(CappedPostmortemCycleDurationMinutesOffsetAdjuster { weight: 0.32 }),
            Rc::new(SourceCappedPostmortemCycleAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.05), ("central".to_string(), 0.93)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 32178.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_postmortem_cycle_engine();
        let item = new_capped_postmortem_cycle("code-947", "label-307");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
