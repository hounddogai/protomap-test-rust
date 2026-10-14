use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default segment notice calculation.
pub trait DefaultSegmentNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultSegmentNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultSegmentNoticeAdjuster {
    pub step: f64,
}

impl DefaultSegmentNoticeAdjuster for RoundingDefaultSegmentNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultSegmentNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultSegmentNoticeAdjuster {
    pub cap: f64,
}

impl DefaultSegmentNoticeAdjuster for CapDefaultSegmentNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultSegmentNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSegmentNoticeDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl DefaultSegmentNoticeAdjuster for DefaultSegmentNoticeDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultSegmentNotice, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultSegmentNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultSegmentNoticeAdjuster for FlaggedDefaultSegmentNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultSegmentNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeDefaultSegmentNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultSegmentNoticeAdjuster for ScopeDefaultSegmentNoticeAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &DefaultSegmentNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default segment notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultSegmentNoticeEngine {
    pub steps: Vec<Rc<dyn DefaultSegmentNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultSegmentNoticeEngine {
    pub fn compute(&mut self, item: &DefaultSegmentNotice) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultSegmentNoticeAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultSegmentNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_segment_notice_engine(flags: &flags::Registry) -> DefaultSegmentNoticeEngine {
    DefaultSegmentNoticeEngine {
        steps: vec![
            Rc::new(RoundingDefaultSegmentNoticeAdjuster { step: 0.5 }) as Rc<dyn DefaultSegmentNoticeAdjuster>,
            Rc::new(CapDefaultSegmentNoticeAdjuster { cap: 22037.0 }),
            Rc::new(DefaultSegmentNoticeDurationMinutesOffsetAdjuster { weight: 0.18 }),
            Rc::new(FlaggedDefaultSegmentNoticeAdjuster {
                flags: flags.clone(),
                flag: "default-segment-notice-default".to_string(),
                percent: 5.3,
            }),
            Rc::new(ScopeDefaultSegmentNoticeAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.17), ("central".to_string(), 1.37)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 67792.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_segment_notice_engine(&flags::new_registry());
        let item = new_default_segment_notice("code-721", "label-734");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
