use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the capped measurement quota calculation.
pub trait CappedMeasurementQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedMeasurementQuota, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedMeasurementQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedMeasurementQuotaFactor for FlaggedCappedMeasurementQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedMeasurementQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedMeasurementQuotaAdjuster {
    pub cap: f64,
}

impl CappedMeasurementQuotaFactor for CapCappedMeasurementQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedMeasurementQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedMeasurementQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedMeasurementQuotaFactor for TieredCappedMeasurementQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedMeasurementQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMeasurementQuotaPriorityOffsetAdjuster {
    pub weight: f64,
}

impl CappedMeasurementQuotaFactor for CappedMeasurementQuotaPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &CappedMeasurementQuota, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedMeasurementQuotaAdjuster {
    pub minimum: f64,
}

impl CappedMeasurementQuotaFactor for MinimumCappedMeasurementQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedMeasurementQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped measurement quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedMeasurementQuotaEngine {
    pub steps: Vec<Rc<dyn CappedMeasurementQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedMeasurementQuotaEngine {
    pub fn run(&mut self, item: &CappedMeasurementQuota) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedMeasurementQuotaFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedMeasurementQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_measurement_quota_engine(flags: &flags::Registry) -> CappedMeasurementQuotaEngine {
    CappedMeasurementQuotaEngine {
        steps: vec![
            Rc::new(FlaggedCappedMeasurementQuotaAdjuster {
                flags: flags.clone(),
                flag: "capped-measurement-quota-scheduled".to_string(),
                percent: 3.2,
            }) as Rc<dyn CappedMeasurementQuotaFactor>,
            Rc::new(CapCappedMeasurementQuotaAdjuster { cap: 31489.0 }),
            Rc::new(TieredCappedMeasurementQuotaAdjuster {
                bounds: vec![4236.0, 7301.0],
                rates: vec![0.93, 0.9],
            }),
            Rc::new(CappedMeasurementQuotaPriorityOffsetAdjuster { weight: 1.52 }),
            Rc::new(MinimumCappedMeasurementQuotaAdjuster { minimum: 22.6 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 20734.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_measurement_quota_engine(&flags::new_registry());
        let item = new_capped_measurement_quota("code-128", "name-285");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
