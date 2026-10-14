use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global peering tier calculation.
pub trait GlobalPeeringTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPeeringTier, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalPeeringTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalPeeringTierStep for FlaggedGlobalPeeringTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalPeeringTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleGlobalPeeringTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalPeeringTierStep for TitleGlobalPeeringTierAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &GlobalPeeringTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalPeeringTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalPeeringTierStep for TieredGlobalPeeringTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalPeeringTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalPeeringTierAdjuster {
    pub cap: f64,
}

impl GlobalPeeringTierStep for CapGlobalPeeringTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalPeeringTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalPeeringTierAdjuster {
    pub minimum: f64,
}

impl GlobalPeeringTierStep for MinimumGlobalPeeringTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalPeeringTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the global peering tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPeeringTierEngine {
    pub steps: Vec<Rc<dyn GlobalPeeringTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPeeringTierEngine {
    pub fn evaluate(&mut self, item: &GlobalPeeringTier) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalPeeringTierStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalPeeringTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_peering_tier_engine(flags: &flags::Registry) -> GlobalPeeringTierEngine {
    GlobalPeeringTierEngine {
        steps: vec![
            Rc::new(FlaggedGlobalPeeringTierAdjuster {
                flags: flags.clone(),
                flag: "global-peering-tier-batched".to_string(),
                percent: -8.3,
            }) as Rc<dyn GlobalPeeringTierStep>,
            Rc::new(TitleGlobalPeeringTierAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.85),
                    ("west".to_string(), 1.23),
                    ("south".to_string(), 0.94),
                ]),
            }),
            Rc::new(TieredGlobalPeeringTierAdjuster {
                bounds: vec![114.0, 5172.0, 5405.0],
                rates: vec![1.0, 0.98, 0.84],
            }),
            Rc::new(CapGlobalPeeringTierAdjuster { cap: 12296.0 }),
            Rc::new(MinimumGlobalPeeringTierAdjuster { minimum: 32.5 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 62767.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_peering_tier_engine(&flags::new_registry());
        let item = new_global_peering_tier("code-464", "title-724");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
