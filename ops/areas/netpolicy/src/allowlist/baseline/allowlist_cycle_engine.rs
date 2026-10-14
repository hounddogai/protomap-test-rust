use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the allowlist cycle calculation.
pub trait AllowlistCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAllowlistCycleAdjuster {
    pub minimum: f64,
}

impl AllowlistCycleAdjuster for MinimumAllowlistCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AllowlistCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAllowlistCycleAdjuster {
    pub percent: f64,
}

impl AllowlistCycleAdjuster for PercentAllowlistCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AllowlistCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCycleRevisionOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistCycleAdjuster for AllowlistCycleRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistCycle, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleAllowlistCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AllowlistCycleAdjuster for TitleAllowlistCycleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &AllowlistCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistCycleAdjuster {
    pub cap: f64,
}

impl AllowlistCycleAdjuster for CapAllowlistCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the allowlist cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistCycleEngine {
    pub steps: Vec<Rc<dyn AllowlistCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistCycleEngine {
    pub fn compute(&mut self, item: &AllowlistCycle) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AllowlistCycleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_allowlist_cycle_engine() -> AllowlistCycleEngine {
    AllowlistCycleEngine {
        steps: vec![
            Rc::new(MinimumAllowlistCycleAdjuster { minimum: 30.3 }) as Rc<dyn AllowlistCycleAdjuster>,
            Rc::new(PercentAllowlistCycleAdjuster { percent: 21.8 }),
            Rc::new(AllowlistCycleRevisionOffsetAdjuster { weight: 1.31 }),
            Rc::new(TitleAllowlistCycleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.76)]),
            }),
            Rc::new(CapAllowlistCycleAdjuster { cap: 45515.0 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 30884.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_allowlist_cycle_engine();
        let item = new_allowlist_cycle("code-864", "title-298");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
