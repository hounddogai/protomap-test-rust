use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the scheduled slo record calculation.
pub trait ScheduledSloRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledSloRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameScheduledSloRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledSloRecordModifier for NameScheduledSloRecordAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ScheduledSloRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRecordRateOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledSloRecordModifier for ScheduledSloRecordRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledSloRecord, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledSloRecordAdjuster {
    pub cap: f64,
}

impl ScheduledSloRecordModifier for CapScheduledSloRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledSloRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled slo record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledSloRecordEngine {
    pub steps: Vec<Rc<dyn ScheduledSloRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledSloRecordEngine {
    pub fn price(&mut self, item: &ScheduledSloRecord) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledSloRecordModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledSloRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_slo_record_engine() -> ScheduledSloRecordEngine {
    ScheduledSloRecordEngine {
        steps: vec![
            Rc::new(NameScheduledSloRecordAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.06), ("south".to_string(), 1.36)]),
            }) as Rc<dyn ScheduledSloRecordModifier>,
            Rc::new(ScheduledSloRecordRateOffsetAdjuster { weight: 1.98 }),
            Rc::new(CapScheduledSloRecordAdjuster { cap: 21669.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 43573.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_slo_record_engine();
        let item = new_scheduled_slo_record("code-684", "name-490");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
