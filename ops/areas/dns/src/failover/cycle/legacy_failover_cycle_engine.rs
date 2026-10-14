use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy failover cycle calculation.
pub trait LegacyFailoverCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyFailoverCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyFailoverCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyFailoverCycleAdjuster for TieredLegacyFailoverCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyFailoverCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLegacyFailoverCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyFailoverCycleAdjuster for TitleLegacyFailoverCycleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LegacyFailoverCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyFailoverCycleAdjuster {
    pub percent: f64,
}

impl LegacyFailoverCycleAdjuster for PercentLegacyFailoverCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyFailoverCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverCycleBudgetOffsetAdjuster {
    pub weight: f64,
}

impl LegacyFailoverCycleAdjuster for LegacyFailoverCycleBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyFailoverCycle, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyFailoverCycleAdjuster {
    pub cap: f64,
}

impl LegacyFailoverCycleAdjuster for CapLegacyFailoverCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyFailoverCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyFailoverCycleAdjuster {
    pub minimum: f64,
}

impl LegacyFailoverCycleAdjuster for MinimumLegacyFailoverCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyFailoverCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy failover cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyFailoverCycleEngine {
    pub steps: Vec<Rc<dyn LegacyFailoverCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyFailoverCycleEngine {
    pub fn price(&mut self, item: &LegacyFailoverCycle) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyFailoverCycleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyFailoverCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_failover_cycle_engine() -> LegacyFailoverCycleEngine {
    LegacyFailoverCycleEngine {
        steps: vec![
            Rc::new(TieredLegacyFailoverCycleAdjuster {
                bounds: vec![2076.0, 2228.0, 3255.0, 3922.0],
                rates: vec![0.97, 0.8, 0.82, 0.8],
            }) as Rc<dyn LegacyFailoverCycleAdjuster>,
            Rc::new(TitleLegacyFailoverCycleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.27)]),
            }),
            Rc::new(PercentLegacyFailoverCycleAdjuster { percent: 21.9 }),
            Rc::new(LegacyFailoverCycleBudgetOffsetAdjuster { weight: 1.53 }),
            Rc::new(CapLegacyFailoverCycleAdjuster { cap: 36903.0 }),
            Rc::new(MinimumLegacyFailoverCycleAdjuster { minimum: 27.8 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 40804.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_failover_cycle_engine();
        let item = new_legacy_failover_cycle("code-208", "title-564");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
