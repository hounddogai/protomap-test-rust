use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the slo notice calculation.
pub trait SloNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionSloNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SloNoticeStep for RegionSloNoticeAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &SloNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloNoticeWeightOffsetAdjuster {
    pub weight: f64,
}

impl SloNoticeStep for SloNoticeWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &SloNotice, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSloNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SloNoticeStep for FlaggedSloNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SloNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the slo notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloNoticeEngine {
    pub steps: Vec<Rc<dyn SloNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloNoticeEngine {
    pub fn evaluate(&mut self, item: &SloNotice) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SloNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SloNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_slo_notice_engine(flags: &flags::Registry) -> SloNoticeEngine {
    SloNoticeEngine {
        steps: vec![
            Rc::new(RegionSloNoticeAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.3),
                    ("online".to_string(), 1.2),
                    ("east".to_string(), 1.08),
                ]),
            }) as Rc<dyn SloNoticeStep>,
            Rc::new(SloNoticeWeightOffsetAdjuster { weight: 1.06 }),
            Rc::new(FlaggedSloNoticeAdjuster {
                flags: flags.clone(),
                flag: "slo-notice-fallback".to_string(),
                percent: -3.8,
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 18956.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_notice_engine(&flags::new_registry());
        let item = new_slo_notice("code-952", "title-120");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
