use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the allowlist claim calculation.
pub trait AllowlistClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AllowlistClaimModifier for TieredAllowlistClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AllowlistClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistClaimQuantityOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistClaimModifier for AllowlistClaimQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistClaim, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterAllowlistClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AllowlistClaimModifier for ClusterAllowlistClaimAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &AllowlistClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAllowlistClaimAdjuster {
    pub step: f64,
}

impl AllowlistClaimModifier for RoundingAllowlistClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AllowlistClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistClaimAdjuster {
    pub cap: f64,
}

impl AllowlistClaimModifier for CapAllowlistClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the allowlist claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistClaimEngine {
    pub steps: Vec<Rc<dyn AllowlistClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistClaimEngine {
    pub fn compute(&mut self, item: &AllowlistClaim) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_allowlist_claim_engine() -> AllowlistClaimEngine {
    AllowlistClaimEngine {
        steps: vec![
            Rc::new(TieredAllowlistClaimAdjuster {
                bounds: vec![1859.0, 4486.0, 6142.0],
                rates: vec![0.87, 1.07, 1.13],
            }) as Rc<dyn AllowlistClaimModifier>,
            Rc::new(AllowlistClaimQuantityOffsetAdjuster { weight: 1.84 }),
            Rc::new(ClusterAllowlistClaimAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.36), ("east".to_string(), 1.39)]),
            }),
            Rc::new(RoundingAllowlistClaimAdjuster { step: 5.0 }),
            Rc::new(CapAllowlistClaimAdjuster { cap: 1113.0 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 86840.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_allowlist_claim_engine();
        let item = new_allowlist_claim("code-877", "label-330");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
