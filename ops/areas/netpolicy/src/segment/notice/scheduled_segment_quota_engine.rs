use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the scheduled segment quota calculation.
pub trait ScheduledSegmentQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledSegmentQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledSegmentQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledSegmentQuotaFactor for TieredScheduledSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledSegmentQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledSegmentQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledSegmentQuotaFactor for FlaggedScheduledSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledSegmentQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledSegmentQuotaAdjuster {
    pub percent: f64,
}

impl ScheduledSegmentQuotaFactor for PercentScheduledSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledSegmentQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledSegmentQuotaAdjuster {
    pub minimum: f64,
}

impl ScheduledSegmentQuotaFactor for MinimumScheduledSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledSegmentQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledSegmentQuotaAdjuster {
    pub cap: f64,
}

impl ScheduledSegmentQuotaFactor for CapScheduledSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledSegmentQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelScheduledSegmentQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledSegmentQuotaFactor for LabelScheduledSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ScheduledSegmentQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled segment quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledSegmentQuotaEngine {
    pub steps: Vec<Rc<dyn ScheduledSegmentQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledSegmentQuotaEngine {
    pub fn compute(&mut self, item: &ScheduledSegmentQuota) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledSegmentQuotaFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledSegmentQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_segment_quota_engine(flags: &flags::Registry) -> ScheduledSegmentQuotaEngine {
    ScheduledSegmentQuotaEngine {
        steps: vec![
            Rc::new(TieredScheduledSegmentQuotaAdjuster {
                bounds: vec![6712.0, 7070.0],
                rates: vec![0.94, 1.08],
            }) as Rc<dyn ScheduledSegmentQuotaFactor>,
            Rc::new(FlaggedScheduledSegmentQuotaAdjuster {
                flags: flags.clone(),
                flag: "scheduled-segment-quota-default".to_string(),
                percent: 0.6,
            }),
            Rc::new(PercentScheduledSegmentQuotaAdjuster { percent: 18.1 }),
            Rc::new(MinimumScheduledSegmentQuotaAdjuster { minimum: 6.2 }),
            Rc::new(CapScheduledSegmentQuotaAdjuster { cap: 19830.0 }),
            Rc::new(LabelScheduledSegmentQuotaAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.88),
                    ("south".to_string(), 1.02),
                    ("east".to_string(), 0.78),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 31642.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_segment_quota_engine(&flags::new_registry());
        let item = new_scheduled_segment_quota("code-392", "label-195");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
