use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the sink notice calculation.
pub trait SinkNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSinkNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkNoticeStep for LabelSinkNoticeAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SinkNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SinkNoticeStep for TieredSinkNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SinkNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkNoticeStep for FlaggedSinkNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkNoticeAdjuster {
    pub percent: f64,
}

impl SinkNoticeStep for PercentSinkNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkNoticeAdjuster {
    pub minimum: f64,
}

impl SinkNoticeStep for MinimumSinkNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSinkNoticeAdjuster {
    pub step: f64,
}

impl SinkNoticeStep for RoundingSinkNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SinkNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the sink notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkNoticeEngine {
    pub steps: Vec<Rc<dyn SinkNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkNoticeEngine {
    pub fn evaluate(&mut self, item: &SinkNotice) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SinkNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SinkNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_sink_notice_engine(flags: &flags::Registry) -> SinkNoticeEngine {
    SinkNoticeEngine {
        steps: vec![
            Rc::new(LabelSinkNoticeAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.17),
                    ("west".to_string(), 1.31),
                    ("online".to_string(), 1.12),
                ]),
            }) as Rc<dyn SinkNoticeStep>,
            Rc::new(TieredSinkNoticeAdjuster {
                bounds: vec![1467.0, 3608.0],
                rates: vec![1.12, 0.99],
            }),
            Rc::new(FlaggedSinkNoticeAdjuster {
                flags: flags.clone(),
                flag: "sink-notice-global".to_string(),
                percent: -2.4,
            }),
            Rc::new(PercentSinkNoticeAdjuster { percent: 3.7 }),
            Rc::new(MinimumSinkNoticeAdjuster { minimum: 47.1 }),
            Rc::new(RoundingSinkNoticeAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 68626.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_notice_engine(&flags::new_registry());
        let item = new_sink_notice("code-673", "label-896");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
