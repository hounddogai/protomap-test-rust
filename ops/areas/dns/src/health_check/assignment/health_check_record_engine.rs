use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the health check record calculation.
pub trait HealthCheckRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckRecord, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckRecordFactor for FlaggedHealthCheckRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentHealthCheckRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckRecordFactor for SegmentHealthCheckRecordAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &HealthCheckRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckRecordFactor for TieredHealthCheckRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the health check record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckRecordEngine {
    pub steps: Vec<Rc<dyn HealthCheckRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckRecordEngine {
    pub fn compute(&mut self, item: &HealthCheckRecord) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn HealthCheckRecordFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_health_check_record_engine(flags: &flags::Registry) -> HealthCheckRecordEngine {
    HealthCheckRecordEngine {
        steps: vec![
            Rc::new(FlaggedHealthCheckRecordAdjuster {
                flags: flags.clone(),
                flag: "health-check-record-primary".to_string(),
                percent: 0.7,
            }) as Rc<dyn HealthCheckRecordFactor>,
            Rc::new(SegmentHealthCheckRecordAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.36)]),
            }),
            Rc::new(TieredHealthCheckRecordAdjuster {
                bounds: vec![2285.0, 8629.0],
                rates: vec![0.9, 0.83],
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 69380.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_health_check_record_engine(&flags::new_registry());
        let item = new_health_check_record("code-774", "name-322");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
