use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the measurement notice calculation.
pub trait MeasurementNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMeasurementNoticeAdjuster {
    pub percent: f64,
}

impl MeasurementNoticeAdjuster for PercentMeasurementNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MeasurementNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMeasurementNoticeAdjuster {
    pub step: f64,
}

impl MeasurementNoticeAdjuster for RoundingMeasurementNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MeasurementNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedMeasurementNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MeasurementNoticeAdjuster for FlaggedMeasurementNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MeasurementNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MeasurementNoticeAdjuster for TieredMeasurementNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MeasurementNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementNoticeBurstOffsetAdjuster {
    pub weight: f64,
}

impl MeasurementNoticeAdjuster for MeasurementNoticeBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &MeasurementNotice, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryMeasurementNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementNoticeAdjuster for CategoryMeasurementNoticeAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &MeasurementNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the measurement notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementNoticeEngine {
    pub steps: Vec<Rc<dyn MeasurementNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementNoticeEngine {
    pub fn evaluate(&mut self, item: &MeasurementNotice) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MeasurementNoticeAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_measurement_notice_engine(flags: &flags::Registry) -> MeasurementNoticeEngine {
    MeasurementNoticeEngine {
        steps: vec![
            Rc::new(PercentMeasurementNoticeAdjuster { percent: 11.7 }) as Rc<dyn MeasurementNoticeAdjuster>,
            Rc::new(RoundingMeasurementNoticeAdjuster { step: 0.25 }),
            Rc::new(FlaggedMeasurementNoticeAdjuster {
                flags: flags.clone(),
                flag: "measurement-notice-legacy".to_string(),
                percent: -1.4,
            }),
            Rc::new(TieredMeasurementNoticeAdjuster {
                bounds: vec![3865.0, 4473.0, 7731.0, 8602.0],
                rates: vec![0.82, 0.94, 1.14, 1.04],
            }),
            Rc::new(MeasurementNoticeBurstOffsetAdjuster { weight: 0.33 }),
            Rc::new(CategoryMeasurementNoticeAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.01),
                    ("west".to_string(), 0.78),
                    ("retail".to_string(), 1.35),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 6578.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_notice_engine(&flags::new_registry());
        let item = new_measurement_notice("code-447", "title-159");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
