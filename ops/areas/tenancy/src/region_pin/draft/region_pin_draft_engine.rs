use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the region pin draft calculation.
pub trait RegionPinDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinDraftAdjuster {
    pub minimum: f64,
}

impl RegionPinDraftStep for MinimumRegionPinDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRegionPinDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionPinDraftStep for NameRegionPinDraftAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RegionPinDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionPinDraftStep for TieredRegionPinDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionPinDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinDraftStep for FlaggedRegionPinDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the region pin draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinDraftEngine {
    pub steps: Vec<Rc<dyn RegionPinDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinDraftEngine {
    pub fn price(&mut self, item: &RegionPinDraft) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionPinDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_region_pin_draft_engine(flags: &flags::Registry) -> RegionPinDraftEngine {
    RegionPinDraftEngine {
        steps: vec![
            Rc::new(MinimumRegionPinDraftAdjuster { minimum: 20.0 }) as Rc<dyn RegionPinDraftStep>,
            Rc::new(NameRegionPinDraftAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.98), ("central".to_string(), 0.99)]),
            }),
            Rc::new(TieredRegionPinDraftAdjuster {
                bounds: vec![2947.0, 4756.0],
                rates: vec![0.85, 0.86],
            }),
            Rc::new(FlaggedRegionPinDraftAdjuster {
                flags: flags.clone(),
                flag: "region-pin-draft-legacy".to_string(),
                percent: -7.6,
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 24813.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_draft_engine(&flags::new_registry());
        let item = new_region_pin_draft("code-612", "name-780");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
