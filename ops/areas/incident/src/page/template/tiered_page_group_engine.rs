use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tiered page group calculation.
pub trait TieredPageGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredPageGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageGroupBudgetOffsetAdjuster {
    pub weight: f64,
}

impl TieredPageGroupAdjuster for TieredPageGroupBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &TieredPageGroup, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredPageGroupAdjuster {
    pub percent: f64,
}

impl TieredPageGroupAdjuster for PercentTieredPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredPageGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleTieredPageGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredPageGroupAdjuster for TitleTieredPageGroupAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &TieredPageGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredPageGroupAdjuster {
    pub cap: f64,
}

impl TieredPageGroupAdjuster for CapTieredPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredPageGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredPageGroupAdjuster {
    pub minimum: f64,
}

impl TieredPageGroupAdjuster for MinimumTieredPageGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredPageGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered page group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredPageGroupEngine {
    pub steps: Vec<Rc<dyn TieredPageGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredPageGroupEngine {
    pub fn run(&mut self, item: &TieredPageGroup) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredPageGroupAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredPageGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_page_group_engine() -> TieredPageGroupEngine {
    TieredPageGroupEngine {
        steps: vec![
            Rc::new(TieredPageGroupBudgetOffsetAdjuster { weight: 1.04 }) as Rc<dyn TieredPageGroupAdjuster>,
            Rc::new(PercentTieredPageGroupAdjuster { percent: 13.9 }),
            Rc::new(TitleTieredPageGroupAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.01),
                    ("west".to_string(), 0.93),
                    ("retail".to_string(), 1.08),
                ]),
            }),
            Rc::new(CapTieredPageGroupAdjuster { cap: 42071.0 }),
            Rc::new(MinimumTieredPageGroupAdjuster { minimum: 17.2 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 70439.5,
    }
}
