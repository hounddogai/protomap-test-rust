use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the suspension tier calculation.
pub trait SuspensionTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTierBudgetOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionTierAdjuster for SuspensionTierBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionTier, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SuspensionTierAdjuster for TieredSuspensionTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SuspensionTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSuspensionTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionTierAdjuster for LabelSuspensionTierAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SuspensionTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSuspensionTierAdjuster {
    pub cap: f64,
}

impl SuspensionTierAdjuster for CapSuspensionTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SuspensionTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionTierAdjuster {
    pub minimum: f64,
}

impl SuspensionTierAdjuster for MinimumSuspensionTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionTierAdjuster {
    pub percent: f64,
}

impl SuspensionTierAdjuster for PercentSuspensionTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the suspension tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionTierEngine {
    pub steps: Vec<Rc<dyn SuspensionTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionTierEngine {
    pub fn run(&mut self, item: &SuspensionTier) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionTierAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_suspension_tier_engine() -> SuspensionTierEngine {
    SuspensionTierEngine {
        steps: vec![
            Rc::new(SuspensionTierBudgetOffsetAdjuster { weight: 1.73 }) as Rc<dyn SuspensionTierAdjuster>,
            Rc::new(TieredSuspensionTierAdjuster {
                bounds: vec![368.0, 2928.0],
                rates: vec![1.03, 0.98],
            }),
            Rc::new(LabelSuspensionTierAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.85)]),
            }),
            Rc::new(CapSuspensionTierAdjuster { cap: 31383.0 }),
            Rc::new(MinimumSuspensionTierAdjuster { minimum: 3.4 }),
            Rc::new(PercentSuspensionTierAdjuster { percent: 2.6 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 94240.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_tier_engine();
        let item = new_suspension_tier("code-119", "label-756");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
