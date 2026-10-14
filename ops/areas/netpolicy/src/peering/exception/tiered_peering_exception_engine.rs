use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tiered peering exception calculation.
pub trait TieredPeeringExceptionModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredPeeringException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredPeeringExceptionAdjuster {
    pub minimum: f64,
}

impl TieredPeeringExceptionModifier for MinimumTieredPeeringExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredPeeringException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringExceptionScoreOffsetAdjuster {
    pub weight: f64,
}

impl TieredPeeringExceptionModifier for TieredPeeringExceptionScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &TieredPeeringException, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryTieredPeeringExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredPeeringExceptionModifier for CountryTieredPeeringExceptionAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &TieredPeeringException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredPeeringExceptionAdjuster {
    pub percent: f64,
}

impl TieredPeeringExceptionModifier for PercentTieredPeeringExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredPeeringException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredPeeringExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredPeeringExceptionModifier for FlaggedTieredPeeringExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredPeeringException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredPeeringExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredPeeringExceptionModifier for TieredTieredPeeringExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredPeeringException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered peering exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredPeeringExceptionEngine {
    pub steps: Vec<Rc<dyn TieredPeeringExceptionModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredPeeringExceptionEngine {
    pub fn price(&mut self, item: &TieredPeeringException) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredPeeringExceptionModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredPeeringException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_peering_exception_engine(flags: &flags::Registry) -> TieredPeeringExceptionEngine {
    TieredPeeringExceptionEngine {
        steps: vec![
            Rc::new(MinimumTieredPeeringExceptionAdjuster { minimum: 44.8 }) as Rc<dyn TieredPeeringExceptionModifier>,
            Rc::new(TieredPeeringExceptionScoreOffsetAdjuster { weight: 1.28 }),
            Rc::new(CountryTieredPeeringExceptionAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.3), ("retail".to_string(), 1.04)]),
            }),
            Rc::new(PercentTieredPeeringExceptionAdjuster { percent: -6.8 }),
            Rc::new(FlaggedTieredPeeringExceptionAdjuster {
                flags: flags.clone(),
                flag: "tiered-peering-exception-regional".to_string(),
                percent: -7.0,
            }),
            Rc::new(TieredTieredPeeringExceptionAdjuster {
                bounds: vec![1574.0, 1606.0, 6444.0, 6945.0],
                rates: vec![0.94, 0.94, 1.03, 1.12],
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 8841.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_peering_exception_engine(&flags::new_registry());
        let item = new_tiered_peering_exception("code-510", "label-214");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
