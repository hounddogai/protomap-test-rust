use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the scheduled dns record segment calculation.
pub trait ScheduledDnsRecordSegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledDnsRecordSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledDnsRecordSegmentAdjuster {
    pub percent: f64,
}

impl ScheduledDnsRecordSegmentFactor for PercentScheduledDnsRecordSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledDnsRecordSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeScheduledDnsRecordSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledDnsRecordSegmentFactor for ScopeScheduledDnsRecordSegmentAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ScheduledDnsRecordSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledDnsRecordSegmentAdjuster {
    pub minimum: f64,
}

impl ScheduledDnsRecordSegmentFactor for MinimumScheduledDnsRecordSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledDnsRecordSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled dns record segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledDnsRecordSegmentEngine {
    pub steps: Vec<Rc<dyn ScheduledDnsRecordSegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledDnsRecordSegmentEngine {
    pub fn compute(&mut self, item: &ScheduledDnsRecordSegment) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledDnsRecordSegmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledDnsRecordSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_dns_record_segment_engine() -> ScheduledDnsRecordSegmentEngine {
    ScheduledDnsRecordSegmentEngine {
        steps: vec![
            Rc::new(PercentScheduledDnsRecordSegmentAdjuster { percent: 19.7 })
                as Rc<dyn ScheduledDnsRecordSegmentFactor>,
            Rc::new(ScopeScheduledDnsRecordSegmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.02), ("retail".to_string(), 1.04)]),
            }),
            Rc::new(MinimumScheduledDnsRecordSegmentAdjuster { minimum: 25.1 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 79589.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_dns_record_segment_engine();
        let item = new_scheduled_dns_record_segment("code-757", "label-970");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
