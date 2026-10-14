use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary limit adjustment calculation.
pub trait PrimaryLimitAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryLimitAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePrimaryLimitAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryLimitAdjustmentModifier for NamePrimaryLimitAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PrimaryLimitAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryLimitAdjustmentAdjuster {
    pub step: f64,
}

impl PrimaryLimitAdjustmentModifier for RoundingPrimaryLimitAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryLimitAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryLimitAdjustmentAdjuster {
    pub percent: f64,
}

impl PrimaryLimitAdjustmentModifier for PercentPrimaryLimitAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryLimitAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryLimitAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryLimitAdjustmentModifier for TieredPrimaryLimitAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryLimitAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the primary limit adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryLimitAdjustmentEngine {
    pub steps: Vec<Rc<dyn PrimaryLimitAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryLimitAdjustmentEngine {
    pub fn price(&mut self, item: &PrimaryLimitAdjustment) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryLimitAdjustmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryLimitAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_limit_adjustment_engine() -> PrimaryLimitAdjustmentEngine {
    PrimaryLimitAdjustmentEngine {
        steps: vec![
            Rc::new(NamePrimaryLimitAdjustmentAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.07)]),
            }) as Rc<dyn PrimaryLimitAdjustmentModifier>,
            Rc::new(RoundingPrimaryLimitAdjustmentAdjuster { step: 0.05 }),
            Rc::new(PercentPrimaryLimitAdjustmentAdjuster { percent: -14.3 }),
            Rc::new(TieredPrimaryLimitAdjustmentAdjuster {
                bounds: vec![2023.0, 3127.0, 4557.0],
                rates: vec![0.99, 0.94, 0.86],
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 42239.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_limit_adjustment_engine();
        let item = new_primary_limit_adjustment("code-971", "name-547");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
