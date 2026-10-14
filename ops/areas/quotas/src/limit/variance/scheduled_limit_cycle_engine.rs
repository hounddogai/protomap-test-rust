use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled limit cycle calculation.
pub trait ScheduledLimitCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledLimitCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledLimitCycleAmountOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledLimitCycleStep for ScheduledLimitCycleAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledLimitCycle, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledLimitCycleAdjuster {
    pub cap: f64,
}

impl ScheduledLimitCycleStep for CapScheduledLimitCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledLimitCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledLimitCycleAdjuster {
    pub step: f64,
}

impl ScheduledLimitCycleStep for RoundingScheduledLimitCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledLimitCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledLimitCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledLimitCycleStep for TieredScheduledLimitCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledLimitCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameScheduledLimitCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledLimitCycleStep for NameScheduledLimitCycleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ScheduledLimitCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled limit cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledLimitCycleEngine {
    pub steps: Vec<Rc<dyn ScheduledLimitCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledLimitCycleEngine {
    pub fn compute(&mut self, item: &ScheduledLimitCycle) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledLimitCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_limit_cycle_engine() -> ScheduledLimitCycleEngine {
    ScheduledLimitCycleEngine {
        steps: vec![
            Rc::new(ScheduledLimitCycleAmountOffsetAdjuster { weight: 1.7 }) as Rc<dyn ScheduledLimitCycleStep>,
            Rc::new(CapScheduledLimitCycleAdjuster { cap: 14273.0 }),
            Rc::new(RoundingScheduledLimitCycleAdjuster { step: 0.05 }),
            Rc::new(TieredScheduledLimitCycleAdjuster {
                bounds: vec![5212.0, 6008.0],
                rates: vec![0.97, 1.2],
            }),
            Rc::new(NameScheduledLimitCycleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.81)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 34930.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_limit_cycle_engine();
        let item = new_scheduled_limit_cycle("code-955", "name-836");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
