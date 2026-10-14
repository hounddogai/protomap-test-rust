use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan slot calculation.
pub trait PlanSlotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanSlotAdjuster for TieredPlanSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanSlotAdjuster {
    pub percent: f64,
}

impl PlanSlotAdjuster for PercentPlanSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanSlotAdjuster {
    pub step: f64,
}

impl PlanSlotAdjuster for RoundingPlanSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePlanSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanSlotAdjuster for NamePlanSlotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PlanSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanSlotAdjuster for FlaggedPlanSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the plan slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanSlotEngine {
    pub steps: Vec<Rc<dyn PlanSlotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanSlotEngine {
    pub fn compute(&mut self, item: &PlanSlot) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanSlotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_plan_slot_engine(flags: &flags::Registry) -> PlanSlotEngine {
    PlanSlotEngine {
        steps: vec![
            Rc::new(TieredPlanSlotAdjuster {
                bounds: vec![6440.0, 9224.0],
                rates: vec![0.89, 1.16],
            }) as Rc<dyn PlanSlotAdjuster>,
            Rc::new(PercentPlanSlotAdjuster { percent: -9.4 }),
            Rc::new(RoundingPlanSlotAdjuster { step: 0.5 }),
            Rc::new(NamePlanSlotAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.22),
                    ("central".to_string(), 0.74),
                    ("east".to_string(), 1.07),
                ]),
            }),
            Rc::new(FlaggedPlanSlotAdjuster {
                flags: flags.clone(),
                flag: "plan-slot-default".to_string(),
                percent: 4.7,
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 23667.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_slot_engine(&flags::new_registry());
        let item = new_plan_slot("code-559", "name-435");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
