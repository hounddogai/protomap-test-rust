use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global failover slot calculation.
pub trait GlobalFailoverSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalFailoverSlot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalFailoverSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalFailoverSlotStep for FlaggedGlobalFailoverSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalFailoverSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFailoverSlotRevisionOffsetAdjuster {
    pub weight: f64,
}

impl GlobalFailoverSlotStep for GlobalFailoverSlotRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalFailoverSlot, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalFailoverSlotAdjuster {
    pub percent: f64,
}

impl GlobalFailoverSlotStep for PercentGlobalFailoverSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalFailoverSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalFailoverSlotAdjuster {
    pub minimum: f64,
}

impl GlobalFailoverSlotStep for MinimumGlobalFailoverSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalFailoverSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalFailoverSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalFailoverSlotStep for TieredGlobalFailoverSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalFailoverSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryGlobalFailoverSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalFailoverSlotStep for CategoryGlobalFailoverSlotAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &GlobalFailoverSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the global failover slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalFailoverSlotEngine {
    pub steps: Vec<Rc<dyn GlobalFailoverSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalFailoverSlotEngine {
    pub fn run(&mut self, item: &GlobalFailoverSlot) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[GlobalFailoverSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_failover_slot_engine(flags: &flags::Registry) -> GlobalFailoverSlotEngine {
    GlobalFailoverSlotEngine {
        steps: vec![
            Rc::new(FlaggedGlobalFailoverSlotAdjuster {
                flags: flags.clone(),
                flag: "global-failover-slot-rolling".to_string(),
                percent: -3.6,
            }) as Rc<dyn GlobalFailoverSlotStep>,
            Rc::new(GlobalFailoverSlotRevisionOffsetAdjuster { weight: 0.89 }),
            Rc::new(PercentGlobalFailoverSlotAdjuster { percent: -3.9 }),
            Rc::new(MinimumGlobalFailoverSlotAdjuster { minimum: 37.4 }),
            Rc::new(TieredGlobalFailoverSlotAdjuster {
                bounds: vec![5648.0, 6512.0],
                rates: vec![0.99, 0.96],
            }),
            Rc::new(CategoryGlobalFailoverSlotAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.07),
                    ("retail".to_string(), 1.32),
                    ("north".to_string(), 0.89),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 2753.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_failover_slot_engine(&flags::new_registry());
        let item = new_global_failover_slot("code-4", "name-800");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
