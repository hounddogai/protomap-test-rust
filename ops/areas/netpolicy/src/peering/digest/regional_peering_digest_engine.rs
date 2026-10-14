use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional peering digest calculation.
pub trait RegionalPeeringDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalPeeringDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRegionalPeeringDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalPeeringDigestAdjuster for NameRegionalPeeringDigestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RegionalPeeringDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalPeeringDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalPeeringDigestAdjuster for TieredRegionalPeeringDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalPeeringDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalPeeringDigestAdjuster {
    pub minimum: f64,
}

impl RegionalPeeringDigestAdjuster for MinimumRegionalPeeringDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalPeeringDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalPeeringDigestAdjuster {
    pub percent: f64,
}

impl RegionalPeeringDigestAdjuster for PercentRegionalPeeringDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalPeeringDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional peering digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalPeeringDigestEngine {
    pub steps: Vec<Rc<dyn RegionalPeeringDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalPeeringDigestEngine {
    pub fn compute(&mut self, item: &RegionalPeeringDigest) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[RegionalPeeringDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_peering_digest_engine() -> RegionalPeeringDigestEngine {
    RegionalPeeringDigestEngine {
        steps: vec![
            Rc::new(NameRegionalPeeringDigestAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.23),
                    ("central".to_string(), 1.34),
                    ("east".to_string(), 0.75),
                ]),
            }) as Rc<dyn RegionalPeeringDigestAdjuster>,
            Rc::new(TieredRegionalPeeringDigestAdjuster {
                bounds: vec![1576.0, 4574.0, 4973.0, 5569.0],
                rates: vec![1.12, 0.9, 0.83, 1.06],
            }),
            Rc::new(MinimumRegionalPeeringDigestAdjuster { minimum: 3.3 }),
            Rc::new(PercentRegionalPeeringDigestAdjuster { percent: -18.6 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 15751.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_peering_digest_engine();
        let item = new_regional_peering_digest("code-240", "name-353");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
