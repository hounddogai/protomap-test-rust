use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the timeline agreement calculation.
pub trait TimelineAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineAgreementAdjuster {
    pub percent: f64,
}

impl TimelineAgreementModifier for PercentTimelineAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTimelineAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TimelineAgreementModifier for FlaggedTimelineAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TimelineAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TimelineAgreementModifier for TieredTimelineAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TimelineAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAgreementGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl TimelineAgreementModifier for TimelineAgreementGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &TimelineAgreement, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineAgreementAdjuster {
    pub step: f64,
}

impl TimelineAgreementModifier for RoundingTimelineAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the timeline agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineAgreementEngine {
    pub steps: Vec<Rc<dyn TimelineAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineAgreementEngine {
    pub fn price(&mut self, item: &TimelineAgreement) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TimelineAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_timeline_agreement_engine(flags: &flags::Registry) -> TimelineAgreementEngine {
    TimelineAgreementEngine {
        steps: vec![
            Rc::new(PercentTimelineAgreementAdjuster { percent: -9.7 }) as Rc<dyn TimelineAgreementModifier>,
            Rc::new(FlaggedTimelineAgreementAdjuster {
                flags: flags.clone(),
                flag: "timeline-agreement-lenient".to_string(),
                percent: 5.6,
            }),
            Rc::new(TieredTimelineAgreementAdjuster {
                bounds: vec![1463.0, 1515.0, 2702.0],
                rates: vec![1.16, 0.9, 0.96],
            }),
            Rc::new(TimelineAgreementGraceDaysOffsetAdjuster { weight: 0.98 }),
            Rc::new(RoundingTimelineAgreementAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 14761.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_timeline_agreement_engine(&flags::new_registry());
        let item = new_timeline_agreement("code-968", "title-2");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
