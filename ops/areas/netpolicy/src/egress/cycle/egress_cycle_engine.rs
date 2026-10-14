use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the egress cycle calculation.
pub trait EgressCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEgressCycleAdjuster {
    pub cap: f64,
}

impl EgressCycleAdjuster for CapEgressCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EgressCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressCycleAdjuster {
    pub percent: f64,
}

impl EgressCycleAdjuster for PercentEgressCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEgressCycleAdjuster {
    pub minimum: f64,
}

impl EgressCycleAdjuster for MinimumEgressCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EgressCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceEgressCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EgressCycleAdjuster for SourceEgressCycleAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &EgressCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the egress cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressCycleEngine {
    pub steps: Vec<Rc<dyn EgressCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressCycleEngine {
    pub fn run(&mut self, item: &EgressCycle) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EgressCycleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EgressCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_egress_cycle_engine() -> EgressCycleEngine {
    EgressCycleEngine {
        steps: vec![
            Rc::new(CapEgressCycleAdjuster { cap: 31499.0 }) as Rc<dyn EgressCycleAdjuster>,
            Rc::new(PercentEgressCycleAdjuster { percent: 14.6 }),
            Rc::new(MinimumEgressCycleAdjuster { minimum: 2.9 }),
            Rc::new(SourceEgressCycleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.31), ("west".to_string(), 0.92)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 78834.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_cycle_engine();
        let item = new_egress_cycle("code-940", "label-333");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
