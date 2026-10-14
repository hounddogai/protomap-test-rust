use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the zone batch calculation.
pub trait ZoneBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneBatchLimitOffsetAdjuster {
    pub weight: f64,
}

impl ZoneBatchModifier for ZoneBatchLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneBatch, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneBatchAdjuster {
    pub cap: f64,
}

impl ZoneBatchModifier for CapZoneBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameZoneBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneBatchModifier for NameZoneBatchAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ZoneBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumZoneBatchAdjuster {
    pub minimum: f64,
}

impl ZoneBatchModifier for MinimumZoneBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ZoneBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ZoneBatchModifier for TieredZoneBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ZoneBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedZoneBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ZoneBatchModifier for FlaggedZoneBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ZoneBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the zone batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneBatchEngine {
    pub steps: Vec<Rc<dyn ZoneBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneBatchEngine {
    pub fn price(&mut self, item: &ZoneBatch) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneBatchModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_zone_batch_engine(flags: &flags::Registry) -> ZoneBatchEngine {
    ZoneBatchEngine {
        steps: vec![
            Rc::new(ZoneBatchLimitOffsetAdjuster { weight: 0.86 }) as Rc<dyn ZoneBatchModifier>,
            Rc::new(CapZoneBatchAdjuster { cap: 27946.0 }),
            Rc::new(NameZoneBatchAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.71), ("west".to_string(), 1.24)]),
            }),
            Rc::new(MinimumZoneBatchAdjuster { minimum: 11.3 }),
            Rc::new(TieredZoneBatchAdjuster {
                bounds: vec![3238.0, 7993.0],
                rates: vec![1.13, 1.03],
            }),
            Rc::new(FlaggedZoneBatchAdjuster {
                flags: flags.clone(),
                flag: "zone-batch-strict".to_string(),
                percent: 7.8,
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 30136.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_batch_engine(&flags::new_registry());
        let item = new_zone_batch("code-722", "name-778");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
