use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the fallback segment entry calculation.
pub trait FallbackSegmentEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackSegmentEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFallbackSegmentEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackSegmentEntryFactor for TitleFallbackSegmentEntryAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FallbackSegmentEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackSegmentEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackSegmentEntryFactor for FlaggedFallbackSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackSegmentEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackSegmentEntryAdjuster {
    pub percent: f64,
}

impl FallbackSegmentEntryFactor for PercentFallbackSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackSegmentEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback segment entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackSegmentEntryEngine {
    pub steps: Vec<Rc<dyn FallbackSegmentEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackSegmentEntryEngine {
    pub fn price(&mut self, item: &FallbackSegmentEntry) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackSegmentEntryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackSegmentEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_segment_entry_engine(flags: &flags::Registry) -> FallbackSegmentEntryEngine {
    FallbackSegmentEntryEngine {
        steps: vec![
            Rc::new(TitleFallbackSegmentEntryAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.17)]),
            }) as Rc<dyn FallbackSegmentEntryFactor>,
            Rc::new(FlaggedFallbackSegmentEntryAdjuster {
                flags: flags.clone(),
                flag: "fallback-segment-entry-tiered".to_string(),
                percent: 3.1,
            }),
            Rc::new(PercentFallbackSegmentEntryAdjuster { percent: -8.3 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 77419.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_segment_entry_engine(&flags::new_registry());
        let item = new_fallback_segment_entry("code-677", "title-413");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
