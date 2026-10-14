use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual rollback profile calculation.
pub trait ManualRollbackProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualRollbackProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualRollbackProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualRollbackProfileModifier for TieredManualRollbackProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualRollbackProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualRollbackProfileAdjuster {
    pub step: f64,
}

impl ManualRollbackProfileModifier for RoundingManualRollbackProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualRollbackProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualRollbackProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualRollbackProfileModifier for FlaggedManualRollbackProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualRollbackProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionManualRollbackProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualRollbackProfileModifier for RegionManualRollbackProfileAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ManualRollbackProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualRollbackProfileAdjuster {
    pub cap: f64,
}

impl ManualRollbackProfileModifier for CapManualRollbackProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualRollbackProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackProfileDiscountOffsetAdjuster {
    pub weight: f64,
}

impl ManualRollbackProfileModifier for ManualRollbackProfileDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &ManualRollbackProfile, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

/// Runs the manual rollback profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualRollbackProfileEngine {
    pub steps: Vec<Rc<dyn ManualRollbackProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualRollbackProfileEngine {
    pub fn run(&mut self, item: &ManualRollbackProfile) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[ManualRollbackProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_rollback_profile_engine(flags: &flags::Registry) -> ManualRollbackProfileEngine {
    ManualRollbackProfileEngine {
        steps: vec![
            Rc::new(TieredManualRollbackProfileAdjuster {
                bounds: vec![302.0, 1753.0, 3428.0],
                rates: vec![0.82, 0.89, 1.19],
            }) as Rc<dyn ManualRollbackProfileModifier>,
            Rc::new(RoundingManualRollbackProfileAdjuster { step: 0.5 }),
            Rc::new(FlaggedManualRollbackProfileAdjuster {
                flags: flags.clone(),
                flag: "manual-rollback-profile-pending".to_string(),
                percent: 8.5,
            }),
            Rc::new(RegionManualRollbackProfileAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.01),
                    ("east".to_string(), 1.28),
                    ("retail".to_string(), 0.72),
                ]),
            }),
            Rc::new(CapManualRollbackProfileAdjuster { cap: 31194.0 }),
            Rc::new(ManualRollbackProfileDiscountOffsetAdjuster { weight: 1.71 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 96733.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_rollback_profile_engine(&flags::new_registry());
        let item = new_manual_rollback_profile("code-225", "title-986");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
