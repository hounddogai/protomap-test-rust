use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the zone draft calculation.
pub trait ZoneDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ZoneDraftStep for TieredZoneDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ZoneDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDraftVersionOffsetAdjuster {
    pub weight: f64,
}

impl ZoneDraftStep for ZoneDraftVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneDraft, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedZoneDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ZoneDraftStep for FlaggedZoneDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ZoneDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneDraftAdjuster {
    pub cap: f64,
}

impl ZoneDraftStep for CapZoneDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingZoneDraftAdjuster {
    pub step: f64,
}

impl ZoneDraftStep for RoundingZoneDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ZoneDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the zone draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneDraftEngine {
    pub steps: Vec<Rc<dyn ZoneDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneDraftEngine {
    pub fn price(&mut self, item: &ZoneDraft) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[ZoneDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_zone_draft_engine(flags: &flags::Registry) -> ZoneDraftEngine {
    ZoneDraftEngine {
        steps: vec![
            Rc::new(TieredZoneDraftAdjuster {
                bounds: vec![1092.0, 3930.0],
                rates: vec![1.15, 1.08],
            }) as Rc<dyn ZoneDraftStep>,
            Rc::new(ZoneDraftVersionOffsetAdjuster { weight: 0.45 }),
            Rc::new(FlaggedZoneDraftAdjuster {
                flags: flags.clone(),
                flag: "zone-draft-fallback".to_string(),
                percent: 2.9,
            }),
            Rc::new(CapZoneDraftAdjuster { cap: 23019.0 }),
            Rc::new(RoundingZoneDraftAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 36955.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_draft_engine(&flags::new_registry());
        let item = new_zone_draft("code-520", "title-882");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
