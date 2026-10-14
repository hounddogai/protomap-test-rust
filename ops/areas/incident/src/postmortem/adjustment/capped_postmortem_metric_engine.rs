use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the capped postmortem metric calculation.
pub trait CappedPostmortemMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedPostmortemMetric, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedPostmortemMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedPostmortemMetricAdjuster for FlaggedCappedPostmortemMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedPostmortemMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedPostmortemMetricAdjuster {
    pub percent: f64,
}

impl CappedPostmortemMetricAdjuster for PercentCappedPostmortemMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedPostmortemMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedPostmortemMetricAdjuster {
    pub cap: f64,
}

impl CappedPostmortemMetricAdjuster for CapCappedPostmortemMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedPostmortemMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryCappedPostmortemMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedPostmortemMetricAdjuster for CategoryCappedPostmortemMetricAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &CappedPostmortemMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemMetricWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl CappedPostmortemMetricAdjuster for CappedPostmortemMetricWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &CappedPostmortemMetric, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

/// Runs the capped postmortem metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedPostmortemMetricEngine {
    pub steps: Vec<Rc<dyn CappedPostmortemMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedPostmortemMetricEngine {
    pub fn price(&mut self, item: &CappedPostmortemMetric) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedPostmortemMetricAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedPostmortemMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_postmortem_metric_engine(flags: &flags::Registry) -> CappedPostmortemMetricEngine {
    CappedPostmortemMetricEngine {
        steps: vec![
            Rc::new(FlaggedCappedPostmortemMetricAdjuster {
                flags: flags.clone(),
                flag: "capped-postmortem-metric-seasonal".to_string(),
                percent: -6.3,
            }) as Rc<dyn CappedPostmortemMetricAdjuster>,
            Rc::new(PercentCappedPostmortemMetricAdjuster { percent: 8.0 }),
            Rc::new(CapCappedPostmortemMetricAdjuster { cap: 5443.0 }),
            Rc::new(CategoryCappedPostmortemMetricAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.38)]),
            }),
            Rc::new(CappedPostmortemMetricWindowDaysOffsetAdjuster { weight: 1.62 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 47484.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_postmortem_metric_engine(&flags::new_registry());
        let item = new_capped_postmortem_metric("code-842", "name-55");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
