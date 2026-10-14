use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback segment window calculation.
pub trait FallbackSegmentWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackSegmentWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentWindowThresholdOffsetAdjuster {
    pub weight: f64,
}

impl FallbackSegmentWindowStep for FallbackSegmentWindowThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackSegmentWindow, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackSegmentWindowAdjuster {
    pub minimum: f64,
}

impl FallbackSegmentWindowStep for MinimumFallbackSegmentWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackSegmentWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackSegmentWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackSegmentWindowStep for TieredFallbackSegmentWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackSegmentWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackSegmentWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackSegmentWindowStep for FlaggedFallbackSegmentWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackSegmentWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryFallbackSegmentWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackSegmentWindowStep for CategoryFallbackSegmentWindowAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &FallbackSegmentWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback segment window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackSegmentWindowEngine {
    pub steps: Vec<Rc<dyn FallbackSegmentWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackSegmentWindowEngine {
    pub fn compute(&mut self, item: &FallbackSegmentWindow) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[FallbackSegmentWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_segment_window_engine(flags: &flags::Registry) -> FallbackSegmentWindowEngine {
    FallbackSegmentWindowEngine {
        steps: vec![
            Rc::new(FallbackSegmentWindowThresholdOffsetAdjuster { weight: 1.9 }) as Rc<dyn FallbackSegmentWindowStep>,
            Rc::new(MinimumFallbackSegmentWindowAdjuster { minimum: 14.1 }),
            Rc::new(TieredFallbackSegmentWindowAdjuster {
                bounds: vec![2600.0, 3809.0, 6125.0, 9222.0],
                rates: vec![0.89, 1.12, 1.12, 0.98],
            }),
            Rc::new(FlaggedFallbackSegmentWindowAdjuster {
                flags: flags.clone(),
                flag: "fallback-segment-window-strict".to_string(),
                percent: 5.0,
            }),
            Rc::new(CategoryFallbackSegmentWindowAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.17),
                    ("east".to_string(), 1.17),
                    ("south".to_string(), 1.08),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 41685.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_segment_window_engine(&flags::new_registry());
        let item = new_fallback_segment_window("code-506", "title-69");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
