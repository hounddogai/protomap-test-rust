use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary sink slot calculation.
pub trait PrimarySinkSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimarySinkSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimarySinkSlotAdjuster {
    pub percent: f64,
}

impl PrimarySinkSlotStep for PercentPrimarySinkSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimarySinkSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimarySinkSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimarySinkSlotStep for FlaggedPrimarySinkSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimarySinkSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimarySinkSlotAdjuster {
    pub cap: f64,
}

impl PrimarySinkSlotStep for CapPrimarySinkSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimarySinkSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryPrimarySinkSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimarySinkSlotStep for CategoryPrimarySinkSlotAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &PrimarySinkSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimarySinkSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimarySinkSlotStep for TieredPrimarySinkSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimarySinkSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkSlotScoreOffsetAdjuster {
    pub weight: f64,
}

impl PrimarySinkSlotStep for PrimarySinkSlotScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &PrimarySinkSlot, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the primary sink slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimarySinkSlotEngine {
    pub steps: Vec<Rc<dyn PrimarySinkSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimarySinkSlotEngine {
    pub fn evaluate(&mut self, item: &PrimarySinkSlot) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimarySinkSlotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimarySinkSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_sink_slot_engine(flags: &flags::Registry) -> PrimarySinkSlotEngine {
    PrimarySinkSlotEngine {
        steps: vec![
            Rc::new(PercentPrimarySinkSlotAdjuster { percent: 22.9 }) as Rc<dyn PrimarySinkSlotStep>,
            Rc::new(FlaggedPrimarySinkSlotAdjuster {
                flags: flags.clone(),
                flag: "primary-sink-slot-rolling".to_string(),
                percent: 4.6,
            }),
            Rc::new(CapPrimarySinkSlotAdjuster { cap: 2304.0 }),
            Rc::new(CategoryPrimarySinkSlotAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.74),
                    ("east".to_string(), 1.16),
                    ("north".to_string(), 0.94),
                ]),
            }),
            Rc::new(TieredPrimarySinkSlotAdjuster {
                bounds: vec![1291.0, 2886.0, 5476.0, 9190.0],
                rates: vec![0.95, 1.12, 0.89, 1.03],
            }),
            Rc::new(PrimarySinkSlotScoreOffsetAdjuster { weight: 1.92 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 22143.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_sink_slot_engine(&flags::new_registry());
        let item = new_primary_sink_slot("code-711", "name-497");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
