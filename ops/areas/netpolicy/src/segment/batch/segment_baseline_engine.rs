use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the segment baseline calculation.
pub trait SegmentBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleSegmentBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SegmentBaselineAdjuster for TitleSegmentBaselineAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &SegmentBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentBaselineAdjuster for TieredSegmentBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBaselineRatioOffsetAdjuster {
    pub weight: f64,
}

impl SegmentBaselineAdjuster for SegmentBaselineRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &SegmentBaseline, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentBaselineAdjuster {
    pub minimum: f64,
}

impl SegmentBaselineAdjuster for MinimumSegmentBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentBaselineAdjuster for FlaggedSegmentBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the segment baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentBaselineEngine {
    pub steps: Vec<Rc<dyn SegmentBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentBaselineEngine {
    pub fn price(&mut self, item: &SegmentBaseline) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SegmentBaselineAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SegmentBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_segment_baseline_engine(flags: &flags::Registry) -> SegmentBaselineEngine {
    SegmentBaselineEngine {
        steps: vec![
            Rc::new(TitleSegmentBaselineAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.13), ("west".to_string(), 1.06)]),
            }) as Rc<dyn SegmentBaselineAdjuster>,
            Rc::new(TieredSegmentBaselineAdjuster {
                bounds: vec![135.0, 8950.0],
                rates: vec![0.97, 0.84],
            }),
            Rc::new(SegmentBaselineRatioOffsetAdjuster { weight: 1.41 }),
            Rc::new(MinimumSegmentBaselineAdjuster { minimum: 25.4 }),
            Rc::new(FlaggedSegmentBaselineAdjuster {
                flags: flags.clone(),
                flag: "segment-baseline-default".to_string(),
                percent: 3.7,
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 62024.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_baseline_engine(&flags::new_registry());
        let item = new_segment_baseline("code-871", "title-821");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
