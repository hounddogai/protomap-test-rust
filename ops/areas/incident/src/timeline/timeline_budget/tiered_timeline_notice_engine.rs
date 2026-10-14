use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered timeline notice calculation.
pub trait TieredTimelineNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredTimelineNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceTieredTimelineNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredTimelineNoticeAdjuster for SourceTieredTimelineNoticeAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &TieredTimelineNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredTimelineNoticeAdjuster {
    pub step: f64,
}

impl TieredTimelineNoticeAdjuster for RoundingTieredTimelineNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredTimelineNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredTimelineNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredTimelineNoticeAdjuster for TieredTieredTimelineNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredTimelineNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered timeline notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredTimelineNoticeEngine {
    pub steps: Vec<Rc<dyn TieredTimelineNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredTimelineNoticeEngine {
    pub fn price(&mut self, item: &TieredTimelineNotice) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[TieredTimelineNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_timeline_notice_engine() -> TieredTimelineNoticeEngine {
    TieredTimelineNoticeEngine {
        steps: vec![
            Rc::new(SourceTieredTimelineNoticeAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.71)]),
            }) as Rc<dyn TieredTimelineNoticeAdjuster>,
            Rc::new(RoundingTieredTimelineNoticeAdjuster { step: 1.0 }),
            Rc::new(TieredTieredTimelineNoticeAdjuster {
                bounds: vec![3917.0, 9034.0],
                rates: vec![0.86, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 31485.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_timeline_notice_engine();
        let item = new_tiered_timeline_notice("code-269", "label-813");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
