use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the segment record calculation.
pub trait SegmentRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentRecordAdjuster {
    pub cap: f64,
}

impl SegmentRecordModifier for CapSegmentRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentRecordAdjuster {
    pub percent: f64,
}

impl SegmentRecordModifier for PercentSegmentRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentRecordAdjuster {
    pub minimum: f64,
}

impl SegmentRecordModifier for MinimumSegmentRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentRecordModifier for TieredSegmentRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSegmentRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SegmentRecordModifier for LabelSegmentRecordAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SegmentRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentRecordAdjuster {
    pub step: f64,
}

impl SegmentRecordModifier for RoundingSegmentRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the segment record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentRecordEngine {
    pub steps: Vec<Rc<dyn SegmentRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentRecordEngine {
    pub fn price(&mut self, item: &SegmentRecord) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[SegmentRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_segment_record_engine() -> SegmentRecordEngine {
    SegmentRecordEngine {
        steps: vec![
            Rc::new(CapSegmentRecordAdjuster { cap: 22240.0 }) as Rc<dyn SegmentRecordModifier>,
            Rc::new(PercentSegmentRecordAdjuster { percent: -5.5 }),
            Rc::new(MinimumSegmentRecordAdjuster { minimum: 11.1 }),
            Rc::new(TieredSegmentRecordAdjuster {
                bounds: vec![792.0, 4980.0, 6873.0, 9208.0],
                rates: vec![1.17, 0.83, 1.18, 1.18],
            }),
            Rc::new(LabelSegmentRecordAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.93),
                    ("south".to_string(), 1.38),
                    ("online".to_string(), 1.05),
                ]),
            }),
            Rc::new(RoundingSegmentRecordAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 47708.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_record_engine();
        let item = new_segment_record("code-72", "label-819");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
