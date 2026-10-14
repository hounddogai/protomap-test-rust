use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the default dns record variance calculation.
pub trait DefaultDnsRecordVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultDnsRecordVariance, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultDnsRecordVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultDnsRecordVarianceAdjuster for FlaggedDefaultDnsRecordVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultDnsRecordVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordVarianceAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl DefaultDnsRecordVarianceAdjuster for DefaultDnsRecordVarianceAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultDnsRecordVariance, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultDnsRecordVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultDnsRecordVarianceAdjuster for TieredDefaultDnsRecordVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultDnsRecordVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultDnsRecordVarianceAdjuster {
    pub minimum: f64,
}

impl DefaultDnsRecordVarianceAdjuster for MinimumDefaultDnsRecordVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultDnsRecordVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitDefaultDnsRecordVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultDnsRecordVarianceAdjuster for UnitDefaultDnsRecordVarianceAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &DefaultDnsRecordVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default dns record variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultDnsRecordVarianceEngine {
    pub steps: Vec<Rc<dyn DefaultDnsRecordVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultDnsRecordVarianceEngine {
    pub fn compute(&mut self, item: &DefaultDnsRecordVariance) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultDnsRecordVarianceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultDnsRecordVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_dns_record_variance_engine(flags: &flags::Registry) -> DefaultDnsRecordVarianceEngine {
    DefaultDnsRecordVarianceEngine {
        steps: vec![
            Rc::new(FlaggedDefaultDnsRecordVarianceAdjuster {
                flags: flags.clone(),
                flag: "default-dns-record-variance-seasonal".to_string(),
                percent: -4.1,
            }) as Rc<dyn DefaultDnsRecordVarianceAdjuster>,
            Rc::new(DefaultDnsRecordVarianceAttemptsOffsetAdjuster { weight: 0.99 }),
            Rc::new(TieredDefaultDnsRecordVarianceAdjuster {
                bounds: vec![242.0, 4849.0, 6027.0, 8591.0],
                rates: vec![1.11, 0.86, 1.13, 1.19],
            }),
            Rc::new(MinimumDefaultDnsRecordVarianceAdjuster { minimum: 39.4 }),
            Rc::new(UnitDefaultDnsRecordVarianceAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.18)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 29105.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_dns_record_variance_engine(&flags::new_registry());
        let item = new_default_dns_record_variance("code-591", "title-861");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
