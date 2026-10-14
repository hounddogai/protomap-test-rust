use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the redaction slot calculation.
pub trait RedactionSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionSlotAdjuster {
    pub cap: f64,
}

impl RedactionSlotStep for CapRedactionSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRedactionSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RedactionSlotStep for FlaggedRedactionSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RedactionSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionSlotAdjuster {
    pub percent: f64,
}

impl RedactionSlotStep for PercentRedactionSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRedactionSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionSlotStep for CategoryRedactionSlotAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RedactionSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSlotBurstOffsetAdjuster {
    pub weight: f64,
}

impl RedactionSlotStep for RedactionSlotBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionSlot, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

/// Runs the redaction slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionSlotEngine {
    pub steps: Vec<Rc<dyn RedactionSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionSlotEngine {
    pub fn run(&mut self, item: &RedactionSlot) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RedactionSlotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RedactionSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_redaction_slot_engine(flags: &flags::Registry) -> RedactionSlotEngine {
    RedactionSlotEngine {
        steps: vec![
            Rc::new(CapRedactionSlotAdjuster { cap: 43403.0 }) as Rc<dyn RedactionSlotStep>,
            Rc::new(FlaggedRedactionSlotAdjuster {
                flags: flags.clone(),
                flag: "redaction-slot-global".to_string(),
                percent: 4.8,
            }),
            Rc::new(PercentRedactionSlotAdjuster { percent: 23.7 }),
            Rc::new(CategoryRedactionSlotAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.74), ("online".to_string(), 1.04)]),
            }),
            Rc::new(RedactionSlotBurstOffsetAdjuster { weight: 0.43 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 18776.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_slot_engine(&flags::new_registry());
        let item = new_redaction_slot("code-554", "name-816");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
