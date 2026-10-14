use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tiered dns record entry calculation.
pub trait TieredDnsRecordEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredDnsRecordEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredDnsRecordEntryAdjuster {
    pub minimum: f64,
}

impl TieredDnsRecordEntryAdjuster for MinimumTieredDnsRecordEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredDnsRecordEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTieredDnsRecordEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredDnsRecordEntryAdjuster for NameTieredDnsRecordEntryAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TieredDnsRecordEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordEntryBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl TieredDnsRecordEntryAdjuster for TieredDnsRecordEntryBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &TieredDnsRecordEntry, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredDnsRecordEntryAdjuster {
    pub cap: f64,
}

impl TieredDnsRecordEntryAdjuster for CapTieredDnsRecordEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredDnsRecordEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the tiered dns record entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredDnsRecordEntryEngine {
    pub steps: Vec<Rc<dyn TieredDnsRecordEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredDnsRecordEntryEngine {
    pub fn run(&mut self, item: &TieredDnsRecordEntry) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredDnsRecordEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredDnsRecordEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_dns_record_entry_engine() -> TieredDnsRecordEntryEngine {
    TieredDnsRecordEntryEngine {
        steps: vec![
            Rc::new(MinimumTieredDnsRecordEntryAdjuster { minimum: 39.5 }) as Rc<dyn TieredDnsRecordEntryAdjuster>,
            Rc::new(NameTieredDnsRecordEntryAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.27), ("online".to_string(), 0.79)]),
            }),
            Rc::new(TieredDnsRecordEntryBatchSizeOffsetAdjuster { weight: 1.36 }),
            Rc::new(CapTieredDnsRecordEntryAdjuster { cap: 23635.0 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 93526.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_dns_record_entry_engine();
        let item = new_tiered_dns_record_entry("code-207", "name-532");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
