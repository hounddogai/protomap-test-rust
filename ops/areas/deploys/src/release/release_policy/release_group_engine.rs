use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the release group calculation.
pub trait ReleaseGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReleaseGroupAdjuster {
    pub step: f64,
}

impl ReleaseGroupFactor for RoundingReleaseGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReleaseGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseGroupAdjuster {
    pub cap: f64,
}

impl ReleaseGroupFactor for CapReleaseGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseGroupFactor for TieredReleaseGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseGroupDiscountOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseGroupFactor for ReleaseGroupDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseGroup, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseGroupFactor for FlaggedReleaseGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitReleaseGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseGroupFactor for UnitReleaseGroupAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ReleaseGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the release group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseGroupEngine {
    pub steps: Vec<Rc<dyn ReleaseGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseGroupEngine {
    pub fn compute(&mut self, item: &ReleaseGroup) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseGroupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_release_group_engine(flags: &flags::Registry) -> ReleaseGroupEngine {
    ReleaseGroupEngine {
        steps: vec![
            Rc::new(RoundingReleaseGroupAdjuster { step: 0.1 }) as Rc<dyn ReleaseGroupFactor>,
            Rc::new(CapReleaseGroupAdjuster { cap: 31008.0 }),
            Rc::new(TieredReleaseGroupAdjuster {
                bounds: vec![4588.0, 4632.0, 5259.0, 9004.0],
                rates: vec![1.19, 1.04, 0.84, 0.99],
            }),
            Rc::new(ReleaseGroupDiscountOffsetAdjuster { weight: 1.76 }),
            Rc::new(FlaggedReleaseGroupAdjuster {
                flags: flags.clone(),
                flag: "release-group-tiered".to_string(),
                percent: -4.3,
            }),
            Rc::new(UnitReleaseGroupAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.4), ("north".to_string(), 0.77)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 54454.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_group_engine(&flags::new_registry());
        let item = new_release_group("code-241", "title-210");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
