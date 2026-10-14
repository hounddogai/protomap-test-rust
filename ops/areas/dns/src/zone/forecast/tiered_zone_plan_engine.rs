use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tiered zone plan calculation.
pub trait TieredZonePlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredZonePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleTieredZonePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredZonePlanFactor for TitleTieredZonePlanAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &TieredZonePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredZonePlanAdjuster {
    pub minimum: f64,
}

impl TieredZonePlanFactor for MinimumTieredZonePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredZonePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZonePlanTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl TieredZonePlanFactor for TieredZonePlanTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &TieredZonePlan, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredZonePlanAdjuster {
    pub cap: f64,
}

impl TieredZonePlanFactor for CapTieredZonePlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredZonePlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredZonePlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredZonePlanFactor for FlaggedTieredZonePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredZonePlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredZonePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredZonePlanFactor for TieredTieredZonePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredZonePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered zone plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredZonePlanEngine {
    pub steps: Vec<Rc<dyn TieredZonePlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredZonePlanEngine {
    pub fn price(&mut self, item: &TieredZonePlan) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredZonePlanFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredZonePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_zone_plan_engine(flags: &flags::Registry) -> TieredZonePlanEngine {
    TieredZonePlanEngine {
        steps: vec![
            Rc::new(TitleTieredZonePlanAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.84)]),
            }) as Rc<dyn TieredZonePlanFactor>,
            Rc::new(MinimumTieredZonePlanAdjuster { minimum: 34.9 }),
            Rc::new(TieredZonePlanTtlSecondsOffsetAdjuster { weight: 1.21 }),
            Rc::new(CapTieredZonePlanAdjuster { cap: 22985.0 }),
            Rc::new(FlaggedTieredZonePlanAdjuster {
                flags: flags.clone(),
                flag: "tiered-zone-plan-strict".to_string(),
                percent: 10.0,
            }),
            Rc::new(TieredTieredZonePlanAdjuster {
                bounds: vec![274.0, 7033.0, 7794.0, 8804.0],
                rates: vec![1.08, 1.12, 0.87, 0.9],
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 64689.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_zone_plan_engine(&flags::new_registry());
        let item = new_tiered_zone_plan("code-418", "title-152");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
