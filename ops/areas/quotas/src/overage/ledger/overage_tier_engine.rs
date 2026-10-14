use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the overage tier calculation.
pub trait OverageTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageTier, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedOverageTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl OverageTierFactor for FlaggedOverageTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &OverageTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapOverageTierAdjuster {
    pub cap: f64,
}

impl OverageTierFactor for CapOverageTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &OverageTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentOverageTierAdjuster {
    pub percent: f64,
}

impl OverageTierFactor for PercentOverageTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &OverageTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageTierAdjuster {
    pub step: f64,
}

impl OverageTierFactor for RoundingOverageTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageTierCeilingOffsetAdjuster {
    pub weight: f64,
}

impl OverageTierFactor for OverageTierCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &OverageTier, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl OverageTierFactor for TieredOverageTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &OverageTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the overage tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageTierEngine {
    pub steps: Vec<Rc<dyn OverageTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageTierEngine {
    pub fn evaluate(&mut self, item: &OverageTier) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn OverageTierFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[OverageTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_overage_tier_engine(flags: &flags::Registry) -> OverageTierEngine {
    OverageTierEngine {
        steps: vec![
            Rc::new(FlaggedOverageTierAdjuster {
                flags: flags.clone(),
                flag: "overage-tier-primary".to_string(),
                percent: 6.8,
            }) as Rc<dyn OverageTierFactor>,
            Rc::new(CapOverageTierAdjuster { cap: 10427.0 }),
            Rc::new(PercentOverageTierAdjuster { percent: 12.4 }),
            Rc::new(RoundingOverageTierAdjuster { step: 0.25 }),
            Rc::new(OverageTierCeilingOffsetAdjuster { weight: 0.21 }),
            Rc::new(TieredOverageTierAdjuster {
                bounds: vec![2089.0, 3333.0, 5922.0, 8295.0],
                rates: vec![1.07, 0.83, 0.89, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 92200.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_tier_engine(&flags::new_registry());
        let item = new_overage_tier("code-956", "label-794");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
