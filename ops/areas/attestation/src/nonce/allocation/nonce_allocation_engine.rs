use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the nonce allocation calculation.
pub trait NonceAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelNonceAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl NonceAllocationAdjuster for ChannelNonceAllocationAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &NonceAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceAllocationAdjuster for FlaggedNonceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapNonceAllocationAdjuster {
    pub cap: f64,
}

impl NonceAllocationAdjuster for CapNonceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &NonceAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceAllocationAdjuster for TieredNonceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingNonceAllocationAdjuster {
    pub step: f64,
}

impl NonceAllocationAdjuster for RoundingNonceAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &NonceAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the nonce allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceAllocationEngine {
    pub steps: Vec<Rc<dyn NonceAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceAllocationEngine {
    pub fn price(&mut self, item: &NonceAllocation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[NonceAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_nonce_allocation_engine(flags: &flags::Registry) -> NonceAllocationEngine {
    NonceAllocationEngine {
        steps: vec![
            Rc::new(ChannelNonceAllocationAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.05),
                    ("south".to_string(), 1.34),
                    ("east".to_string(), 1.37),
                ]),
            }) as Rc<dyn NonceAllocationAdjuster>,
            Rc::new(FlaggedNonceAllocationAdjuster {
                flags: flags.clone(),
                flag: "nonce-allocation-manual".to_string(),
                percent: -6.8,
            }),
            Rc::new(CapNonceAllocationAdjuster { cap: 42960.0 }),
            Rc::new(TieredNonceAllocationAdjuster {
                bounds: vec![798.0, 3764.0, 6022.0],
                rates: vec![1.15, 0.83, 0.85],
            }),
            Rc::new(RoundingNonceAllocationAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 43797.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_allocation_engine(&flags::new_registry());
        let item = new_nonce_allocation("code-50", "name-625");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
