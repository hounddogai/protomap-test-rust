use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the grant segment calculation.
pub trait GrantSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantSegment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGrantSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GrantSegmentStep for FlaggedGrantSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GrantSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGrantSegmentAdjuster {
    pub cap: f64,
}

impl GrantSegmentStep for CapGrantSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GrantSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelGrantSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantSegmentStep for LabelGrantSegmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &GrantSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantSegmentAdjuster {
    pub minimum: f64,
}

impl GrantSegmentStep for MinimumGrantSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GrantSegmentStep for TieredGrantSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GrantSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the grant segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantSegmentEngine {
    pub steps: Vec<Rc<dyn GrantSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantSegmentEngine {
    pub fn compute(&mut self, item: &GrantSegment) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GrantSegmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GrantSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_grant_segment_engine(flags: &flags::Registry) -> GrantSegmentEngine {
    GrantSegmentEngine {
        steps: vec![
            Rc::new(FlaggedGrantSegmentAdjuster {
                flags: flags.clone(),
                flag: "grant-segment-global".to_string(),
                percent: -10.0,
            }) as Rc<dyn GrantSegmentStep>,
            Rc::new(CapGrantSegmentAdjuster { cap: 36193.0 }),
            Rc::new(LabelGrantSegmentAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.31), ("central".to_string(), 0.76)]),
            }),
            Rc::new(MinimumGrantSegmentAdjuster { minimum: 16.5 }),
            Rc::new(TieredGrantSegmentAdjuster {
                bounds: vec![1656.0, 2405.0, 8464.0],
                rates: vec![1.13, 0.9, 1.06],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 22773.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_segment_engine(&flags::new_registry());
        let item = new_grant_segment("code-105", "label-584");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
