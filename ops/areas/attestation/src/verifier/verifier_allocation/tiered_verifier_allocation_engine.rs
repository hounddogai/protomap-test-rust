use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered verifier allocation calculation.
pub trait TieredVerifierAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredVerifierAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierAllocationCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl TieredVerifierAllocationAdjuster for TieredVerifierAllocationCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &TieredVerifierAllocation, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredVerifierAllocationAdjuster {
    pub cap: f64,
}

impl TieredVerifierAllocationAdjuster for CapTieredVerifierAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredVerifierAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredVerifierAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredVerifierAllocationAdjuster for FlaggedTieredVerifierAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredVerifierAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredVerifierAllocationAdjuster {
    pub minimum: f64,
}

impl TieredVerifierAllocationAdjuster for MinimumTieredVerifierAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredVerifierAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTieredVerifierAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredVerifierAllocationAdjuster for NameTieredVerifierAllocationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TieredVerifierAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the tiered verifier allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredVerifierAllocationEngine {
    pub steps: Vec<Rc<dyn TieredVerifierAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredVerifierAllocationEngine {
    pub fn price(&mut self, item: &TieredVerifierAllocation) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[TieredVerifierAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_verifier_allocation_engine(flags: &flags::Registry) -> TieredVerifierAllocationEngine {
    TieredVerifierAllocationEngine {
        steps: vec![
            Rc::new(TieredVerifierAllocationCreatedAtOffsetAdjuster { weight: 1.04 })
                as Rc<dyn TieredVerifierAllocationAdjuster>,
            Rc::new(CapTieredVerifierAllocationAdjuster { cap: 27035.0 }),
            Rc::new(FlaggedTieredVerifierAllocationAdjuster {
                flags: flags.clone(),
                flag: "tiered-verifier-allocation-primary".to_string(),
                percent: 8.9,
            }),
            Rc::new(MinimumTieredVerifierAllocationAdjuster { minimum: 9.6 }),
            Rc::new(NameTieredVerifierAllocationAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.8)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 75965.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_verifier_allocation_engine(&flags::new_registry());
        let item = new_tiered_verifier_allocation("code-85", "name-518");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
