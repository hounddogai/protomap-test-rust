use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending segment preference calculation.
pub trait PendingSegmentPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingSegmentPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingSegmentPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingSegmentPreferenceAdjuster for TieredPendingSegmentPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingSegmentPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingSegmentPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingSegmentPreferenceAdjuster for FlaggedPendingSegmentPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingSegmentPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePendingSegmentPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingSegmentPreferenceAdjuster for TitlePendingSegmentPreferenceAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PendingSegmentPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingSegmentPreferenceAdjuster {
    pub cap: f64,
}

impl PendingSegmentPreferenceAdjuster for CapPendingSegmentPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingSegmentPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingSegmentPreferenceAdjuster {
    pub minimum: f64,
}

impl PendingSegmentPreferenceAdjuster for MinimumPendingSegmentPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingSegmentPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending segment preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingSegmentPreferenceEngine {
    pub steps: Vec<Rc<dyn PendingSegmentPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingSegmentPreferenceEngine {
    pub fn run(&mut self, item: &PendingSegmentPreference) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[PendingSegmentPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_segment_preference_engine(flags: &flags::Registry) -> PendingSegmentPreferenceEngine {
    PendingSegmentPreferenceEngine {
        steps: vec![
            Rc::new(TieredPendingSegmentPreferenceAdjuster {
                bounds: vec![3613.0, 8106.0],
                rates: vec![0.96, 0.83],
            }) as Rc<dyn PendingSegmentPreferenceAdjuster>,
            Rc::new(FlaggedPendingSegmentPreferenceAdjuster {
                flags: flags.clone(),
                flag: "pending-segment-preference-scheduled".to_string(),
                percent: 6.7,
            }),
            Rc::new(TitlePendingSegmentPreferenceAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.3)]),
            }),
            Rc::new(CapPendingSegmentPreferenceAdjuster { cap: 13566.0 }),
            Rc::new(MinimumPendingSegmentPreferenceAdjuster { minimum: 22.5 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 88628.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_segment_preference_engine(&flags::new_registry());
        let item = new_pending_segment_preference("code-745", "title-934");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
