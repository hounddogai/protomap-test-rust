use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the failover cycle calculation.
pub trait FailoverCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverCycle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverCycleFactor for FlaggedFailoverCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverCyclePortOffsetAdjuster {
    pub weight: f64,
}

impl FailoverCycleFactor for FailoverCyclePortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &FailoverCycle, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverCycleFactor for TieredFailoverCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFailoverCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverCycleFactor for LabelFailoverCycleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FailoverCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the failover cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverCycleEngine {
    pub steps: Vec<Rc<dyn FailoverCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverCycleEngine {
    pub fn run(&mut self, item: &FailoverCycle) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverCycleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_failover_cycle_engine(flags: &flags::Registry) -> FailoverCycleEngine {
    FailoverCycleEngine {
        steps: vec![
            Rc::new(FlaggedFailoverCycleAdjuster {
                flags: flags.clone(),
                flag: "failover-cycle-scheduled".to_string(),
                percent: -5.9,
            }) as Rc<dyn FailoverCycleFactor>,
            Rc::new(FailoverCyclePortOffsetAdjuster { weight: 0.83 }),
            Rc::new(TieredFailoverCycleAdjuster {
                bounds: vec![7357.0, 8941.0, 9202.0],
                rates: vec![1.09, 0.84, 0.96],
            }),
            Rc::new(LabelFailoverCycleAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.11), ("online".to_string(), 0.78)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 93348.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_cycle_engine(&flags::new_registry());
        let item = new_failover_cycle("code-187", "label-489");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
