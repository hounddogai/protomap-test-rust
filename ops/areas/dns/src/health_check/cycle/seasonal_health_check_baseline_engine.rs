use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the seasonal health check baseline calculation.
pub trait SeasonalHealthCheckBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalHealthCheckBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalHealthCheckBaselineAdjuster {
    pub cap: f64,
}

impl SeasonalHealthCheckBaselineAdjuster for CapSeasonalHealthCheckBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalHealthCheckBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSeasonalHealthCheckBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalHealthCheckBaselineAdjuster for SegmentSeasonalHealthCheckBaselineAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &SeasonalHealthCheckBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalHealthCheckBaselineAdjuster {
    pub minimum: f64,
}

impl SeasonalHealthCheckBaselineAdjuster for MinimumSeasonalHealthCheckBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalHealthCheckBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalHealthCheckBaselineAdjuster {
    pub percent: f64,
}

impl SeasonalHealthCheckBaselineAdjuster for PercentSeasonalHealthCheckBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalHealthCheckBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckBaselineSequenceOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalHealthCheckBaselineAdjuster for SeasonalHealthCheckBaselineSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalHealthCheckBaseline, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

/// Runs the seasonal health check baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalHealthCheckBaselineEngine {
    pub steps: Vec<Rc<dyn SeasonalHealthCheckBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalHealthCheckBaselineEngine {
    pub fn run(&mut self, item: &SeasonalHealthCheckBaseline) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalHealthCheckBaselineAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalHealthCheckBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_health_check_baseline_engine() -> SeasonalHealthCheckBaselineEngine {
    SeasonalHealthCheckBaselineEngine {
        steps: vec![
            Rc::new(CapSeasonalHealthCheckBaselineAdjuster { cap: 40780.0 })
                as Rc<dyn SeasonalHealthCheckBaselineAdjuster>,
            Rc::new(SegmentSeasonalHealthCheckBaselineAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.17), ("south".to_string(), 0.91)]),
            }),
            Rc::new(MinimumSeasonalHealthCheckBaselineAdjuster { minimum: 12.7 }),
            Rc::new(PercentSeasonalHealthCheckBaselineAdjuster { percent: -3.8 }),
            Rc::new(SeasonalHealthCheckBaselineSequenceOffsetAdjuster { weight: 0.81 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 40242.8,
    }
}
