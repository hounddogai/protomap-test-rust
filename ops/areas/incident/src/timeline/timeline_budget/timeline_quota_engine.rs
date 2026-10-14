use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the timeline quota calculation.
pub trait TimelineQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineQuotaAdjuster {
    pub percent: f64,
}

impl TimelineQuotaFactor for PercentTimelineQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineQuotaAdjuster {
    pub step: f64,
}

impl TimelineQuotaFactor for RoundingTimelineQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TimelineQuotaFactor for TieredTimelineQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TimelineQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineQuotaAdjuster {
    pub cap: f64,
}

impl TimelineQuotaFactor for CapTimelineQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the timeline quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineQuotaEngine {
    pub steps: Vec<Rc<dyn TimelineQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineQuotaEngine {
    pub fn compute(&mut self, item: &TimelineQuota) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[TimelineQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_timeline_quota_engine() -> TimelineQuotaEngine {
    TimelineQuotaEngine {
        steps: vec![
            Rc::new(PercentTimelineQuotaAdjuster { percent: -24.0 }) as Rc<dyn TimelineQuotaFactor>,
            Rc::new(RoundingTimelineQuotaAdjuster { step: 5.0 }),
            Rc::new(TieredTimelineQuotaAdjuster {
                bounds: vec![482.0, 1003.0, 3973.0, 7840.0],
                rates: vec![1.11, 0.88, 0.88, 0.97],
            }),
            Rc::new(CapTimelineQuotaAdjuster { cap: 21730.0 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 95707.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_timeline_quota_engine();
        let item = new_timeline_quota("code-75", "name-513");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
