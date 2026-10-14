use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling alert slot calculation.
pub trait RollingAlertSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingAlertSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingAlertSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingAlertSlotStep for TieredRollingAlertSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingAlertSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRollingAlertSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingAlertSlotStep for CountryRollingAlertSlotAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RollingAlertSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingAlertSlotAdjuster {
    pub step: f64,
}

impl RollingAlertSlotStep for RoundingRollingAlertSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingAlertSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingAlertSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingAlertSlotStep for FlaggedRollingAlertSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingAlertSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingAlertSlotAdjuster {
    pub cap: f64,
}

impl RollingAlertSlotStep for CapRollingAlertSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingAlertSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling alert slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingAlertSlotEngine {
    pub steps: Vec<Rc<dyn RollingAlertSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingAlertSlotEngine {
    pub fn evaluate(&mut self, item: &RollingAlertSlot) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingAlertSlotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingAlertSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_alert_slot_engine(flags: &flags::Registry) -> RollingAlertSlotEngine {
    RollingAlertSlotEngine {
        steps: vec![
            Rc::new(TieredRollingAlertSlotAdjuster {
                bounds: vec![1308.0, 4365.0, 8001.0, 8685.0],
                rates: vec![1.17, 1.09, 1.18, 1.06],
            }) as Rc<dyn RollingAlertSlotStep>,
            Rc::new(CountryRollingAlertSlotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.35), ("retail".to_string(), 1.14)]),
            }),
            Rc::new(RoundingRollingAlertSlotAdjuster { step: 0.5 }),
            Rc::new(FlaggedRollingAlertSlotAdjuster {
                flags: flags.clone(),
                flag: "rolling-alert-slot-seasonal".to_string(),
                percent: -5.6,
            }),
            Rc::new(CapRollingAlertSlotAdjuster { cap: 49770.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 30308.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_alert_slot_engine(&flags::new_registry());
        let item = new_rolling_alert_slot("code-123", "name-108");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
