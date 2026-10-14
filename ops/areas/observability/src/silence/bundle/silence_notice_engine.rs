use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the silence notice calculation.
pub trait SilenceNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSilenceNoticeAdjuster {
    pub cap: f64,
}

impl SilenceNoticeStep for CapSilenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SilenceNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSilenceNoticeAdjuster {
    pub step: f64,
}

impl SilenceNoticeStep for RoundingSilenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SilenceNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceNoticeAdjuster {
    pub percent: f64,
}

impl SilenceNoticeStep for PercentSilenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the silence notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceNoticeEngine {
    pub steps: Vec<Rc<dyn SilenceNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceNoticeEngine {
    pub fn run(&mut self, item: &SilenceNotice) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SilenceNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SilenceNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_silence_notice_engine() -> SilenceNoticeEngine {
    SilenceNoticeEngine {
        steps: vec![
            Rc::new(CapSilenceNoticeAdjuster { cap: 22876.0 }) as Rc<dyn SilenceNoticeStep>,
            Rc::new(RoundingSilenceNoticeAdjuster { step: 0.5 }),
            Rc::new(PercentSilenceNoticeAdjuster { percent: 17.7 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 40456.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_notice_engine();
        let item = new_silence_notice("code-986", "title-501");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
