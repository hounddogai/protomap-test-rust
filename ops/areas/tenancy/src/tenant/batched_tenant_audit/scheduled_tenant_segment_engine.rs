use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled tenant segment calculation.
pub trait ScheduledTenantSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledTenantSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledTenantSegmentAdjuster {
    pub minimum: f64,
}

impl ScheduledTenantSegmentStep for MinimumScheduledTenantSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledTenantSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledTenantSegmentAdjuster {
    pub step: f64,
}

impl ScheduledTenantSegmentStep for RoundingScheduledTenantSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledTenantSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledTenantSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledTenantSegmentStep for FlaggedScheduledTenantSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledTenantSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScheduledTenantSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledTenantSegmentStep for SegmentScheduledTenantSegmentAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ScheduledTenantSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled tenant segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledTenantSegmentEngine {
    pub steps: Vec<Rc<dyn ScheduledTenantSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledTenantSegmentEngine {
    pub fn price(&mut self, item: &ScheduledTenantSegment) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledTenantSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_tenant_segment_engine(flags: &flags::Registry) -> ScheduledTenantSegmentEngine {
    ScheduledTenantSegmentEngine {
        steps: vec![
            Rc::new(MinimumScheduledTenantSegmentAdjuster { minimum: 29.1 }) as Rc<dyn ScheduledTenantSegmentStep>,
            Rc::new(RoundingScheduledTenantSegmentAdjuster { step: 1.0 }),
            Rc::new(FlaggedScheduledTenantSegmentAdjuster {
                flags: flags.clone(),
                flag: "scheduled-tenant-segment-capped".to_string(),
                percent: 0.2,
            }),
            Rc::new(SegmentScheduledTenantSegmentAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.71)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 72218.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_tenant_segment_engine(&flags::new_registry());
        let item = new_scheduled_tenant_segment("code-660", "title-743");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
