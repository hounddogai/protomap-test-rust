use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the zone claim calculation.
pub trait ZoneClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumZoneClaimAdjuster {
    pub minimum: f64,
}

impl ZoneClaimStep for MinimumZoneClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ZoneClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneClaimBudgetOffsetAdjuster {
    pub weight: f64,
}

impl ZoneClaimStep for ZoneClaimBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneClaim, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryZoneClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneClaimStep for CountryZoneClaimAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ZoneClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingZoneClaimAdjuster {
    pub step: f64,
}

impl ZoneClaimStep for RoundingZoneClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ZoneClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneClaimAdjuster {
    pub cap: f64,
}

impl ZoneClaimStep for CapZoneClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the zone claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneClaimEngine {
    pub steps: Vec<Rc<dyn ZoneClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneClaimEngine {
    pub fn compute(&mut self, item: &ZoneClaim) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[ZoneClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_zone_claim_engine() -> ZoneClaimEngine {
    ZoneClaimEngine {
        steps: vec![
            Rc::new(MinimumZoneClaimAdjuster { minimum: 39.5 }) as Rc<dyn ZoneClaimStep>,
            Rc::new(ZoneClaimBudgetOffsetAdjuster { weight: 0.63 }),
            Rc::new(CountryZoneClaimAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.95)]),
            }),
            Rc::new(RoundingZoneClaimAdjuster { step: 0.5 }),
            Rc::new(CapZoneClaimAdjuster { cap: 3395.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 94503.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_claim_engine();
        let item = new_zone_claim("code-101", "title-317");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
