use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled identity quota calculation.
pub trait ScheduledIdentityQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledIdentityQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleScheduledIdentityQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledIdentityQuotaFactor for TitleScheduledIdentityQuotaAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ScheduledIdentityQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledIdentityQuotaAdjuster {
    pub percent: f64,
}

impl ScheduledIdentityQuotaFactor for PercentScheduledIdentityQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledIdentityQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledIdentityQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledIdentityQuotaFactor for TieredScheduledIdentityQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledIdentityQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledIdentityQuotaAdjuster {
    pub step: f64,
}

impl ScheduledIdentityQuotaFactor for RoundingScheduledIdentityQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledIdentityQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledIdentityQuotaAdjuster {
    pub cap: f64,
}

impl ScheduledIdentityQuotaFactor for CapScheduledIdentityQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledIdentityQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledIdentityQuotaAdjuster {
    pub minimum: f64,
}

impl ScheduledIdentityQuotaFactor for MinimumScheduledIdentityQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledIdentityQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled identity quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledIdentityQuotaEngine {
    pub steps: Vec<Rc<dyn ScheduledIdentityQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledIdentityQuotaEngine {
    pub fn price(&mut self, item: &ScheduledIdentityQuota) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledIdentityQuotaFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledIdentityQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_identity_quota_engine() -> ScheduledIdentityQuotaEngine {
    ScheduledIdentityQuotaEngine {
        steps: vec![
            Rc::new(TitleScheduledIdentityQuotaAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.84),
                    ("west".to_string(), 1.33),
                    ("south".to_string(), 0.83),
                ]),
            }) as Rc<dyn ScheduledIdentityQuotaFactor>,
            Rc::new(PercentScheduledIdentityQuotaAdjuster { percent: 14.4 }),
            Rc::new(TieredScheduledIdentityQuotaAdjuster {
                bounds: vec![1201.0, 3028.0, 4482.0, 8672.0],
                rates: vec![0.88, 0.95, 0.88, 0.92],
            }),
            Rc::new(RoundingScheduledIdentityQuotaAdjuster { step: 0.25 }),
            Rc::new(CapScheduledIdentityQuotaAdjuster { cap: 43177.0 }),
            Rc::new(MinimumScheduledIdentityQuotaAdjuster { minimum: 43.0 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 33855.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_identity_quota_engine();
        let item = new_scheduled_identity_quota("code-646", "title-11");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
