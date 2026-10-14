use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global binding segment calculation.
pub trait GlobalBindingSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalBindingSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalBindingSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalBindingSegmentStep for TieredGlobalBindingSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalBindingSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalBindingSegmentAdjuster {
    pub cap: f64,
}

impl GlobalBindingSegmentStep for CapGlobalBindingSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalBindingSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalBindingSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalBindingSegmentStep for FlaggedGlobalBindingSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalBindingSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeGlobalBindingSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalBindingSegmentStep for ScopeGlobalBindingSegmentAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &GlobalBindingSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalBindingSegmentAdjuster {
    pub minimum: f64,
}

impl GlobalBindingSegmentStep for MinimumGlobalBindingSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalBindingSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalBindingSegmentAdjuster {
    pub percent: f64,
}

impl GlobalBindingSegmentStep for PercentGlobalBindingSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalBindingSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global binding segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalBindingSegmentEngine {
    pub steps: Vec<Rc<dyn GlobalBindingSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalBindingSegmentEngine {
    pub fn price(&mut self, item: &GlobalBindingSegment) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalBindingSegmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalBindingSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_binding_segment_engine(flags: &flags::Registry) -> GlobalBindingSegmentEngine {
    GlobalBindingSegmentEngine {
        steps: vec![
            Rc::new(TieredGlobalBindingSegmentAdjuster {
                bounds: vec![5772.0, 7027.0],
                rates: vec![1.04, 0.87],
            }) as Rc<dyn GlobalBindingSegmentStep>,
            Rc::new(CapGlobalBindingSegmentAdjuster { cap: 27718.0 }),
            Rc::new(FlaggedGlobalBindingSegmentAdjuster {
                flags: flags.clone(),
                flag: "global-binding-segment-manual".to_string(),
                percent: -8.2,
            }),
            Rc::new(ScopeGlobalBindingSegmentAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.03)]),
            }),
            Rc::new(MinimumGlobalBindingSegmentAdjuster { minimum: 8.0 }),
            Rc::new(PercentGlobalBindingSegmentAdjuster { percent: 14.5 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 25982.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_binding_segment_engine(&flags::new_registry());
        let item = new_global_binding_segment("code-634", "title-758");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
