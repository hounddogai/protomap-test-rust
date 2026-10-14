use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default sink notice calculation.
pub trait DefaultSinkNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultSinkNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultSinkNoticeAdjuster {
    pub minimum: f64,
}

impl DefaultSinkNoticeStep for MinimumDefaultSinkNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultSinkNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultSinkNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultSinkNoticeStep for FlaggedDefaultSinkNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultSinkNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDefaultSinkNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultSinkNoticeStep for NameDefaultSinkNoticeAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DefaultSinkNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultSinkNoticeAdjuster {
    pub step: f64,
}

impl DefaultSinkNoticeStep for RoundingDefaultSinkNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultSinkNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the default sink notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultSinkNoticeEngine {
    pub steps: Vec<Rc<dyn DefaultSinkNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultSinkNoticeEngine {
    pub fn evaluate(&mut self, item: &DefaultSinkNotice) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultSinkNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultSinkNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_sink_notice_engine(flags: &flags::Registry) -> DefaultSinkNoticeEngine {
    DefaultSinkNoticeEngine {
        steps: vec![
            Rc::new(MinimumDefaultSinkNoticeAdjuster { minimum: 40.6 }) as Rc<dyn DefaultSinkNoticeStep>,
            Rc::new(FlaggedDefaultSinkNoticeAdjuster {
                flags: flags.clone(),
                flag: "default-sink-notice-global".to_string(),
                percent: -6.2,
            }),
            Rc::new(NameDefaultSinkNoticeAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.86),
                    ("east".to_string(), 0.95),
                    ("central".to_string(), 1.27),
                ]),
            }),
            Rc::new(RoundingDefaultSinkNoticeAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 51252.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_sink_notice_engine(&flags::new_registry());
        let item = new_default_sink_notice("code-698", "name-228");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
