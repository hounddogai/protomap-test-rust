use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dns record metric calculation.
pub trait DnsRecordMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordMetricAdjuster {
    pub minimum: f64,
}

impl DnsRecordMetricStep for MinimumDnsRecordMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordMetricStep for TieredDnsRecordMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordMetricStep for FlaggedDnsRecordMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDnsRecordMetricAdjuster {
    pub step: f64,
}

impl DnsRecordMetricStep for RoundingDnsRecordMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DnsRecordMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDnsRecordMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordMetricStep for NameDnsRecordMetricAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DnsRecordMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the dns record metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordMetricEngine {
    pub steps: Vec<Rc<dyn DnsRecordMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordMetricEngine {
    pub fn run(&mut self, item: &DnsRecordMetric) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordMetricStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_dns_record_metric_engine(flags: &flags::Registry) -> DnsRecordMetricEngine {
    DnsRecordMetricEngine {
        steps: vec![
            Rc::new(MinimumDnsRecordMetricAdjuster { minimum: 40.7 }) as Rc<dyn DnsRecordMetricStep>,
            Rc::new(TieredDnsRecordMetricAdjuster {
                bounds: vec![5759.0, 5768.0],
                rates: vec![1.19, 1.04],
            }),
            Rc::new(FlaggedDnsRecordMetricAdjuster {
                flags: flags.clone(),
                flag: "dns-record-metric-default".to_string(),
                percent: -5.3,
            }),
            Rc::new(RoundingDnsRecordMetricAdjuster { step: 0.5 }),
            Rc::new(NameDnsRecordMetricAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.76)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 44501.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_metric_engine(&flags::new_registry());
        let item = new_dns_record_metric("code-103", "name-747");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
