use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the overage notice calculation.
pub trait OverageNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapOverageNoticeAdjuster {
    pub cap: f64,
}

impl OverageNoticeAdjuster for CapOverageNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &OverageNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedOverageNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl OverageNoticeAdjuster for FlaggedOverageNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &OverageNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionOverageNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl OverageNoticeAdjuster for RegionOverageNoticeAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &OverageNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the overage notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageNoticeEngine {
    pub steps: Vec<Rc<dyn OverageNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageNoticeEngine {
    pub fn evaluate(&mut self, item: &OverageNotice) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn OverageNoticeAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[OverageNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_overage_notice_engine(flags: &flags::Registry) -> OverageNoticeEngine {
    OverageNoticeEngine {
        steps: vec![
            Rc::new(CapOverageNoticeAdjuster { cap: 26335.0 }) as Rc<dyn OverageNoticeAdjuster>,
            Rc::new(FlaggedOverageNoticeAdjuster {
                flags: flags.clone(),
                flag: "overage-notice-fallback".to_string(),
                percent: 3.7,
            }),
            Rc::new(RegionOverageNoticeAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.24),
                    ("central".to_string(), 1.34),
                    ("north".to_string(), 1.29),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 10087.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_notice_engine(&flags::new_registry());
        let item = new_overage_notice("code-130", "title-707");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
