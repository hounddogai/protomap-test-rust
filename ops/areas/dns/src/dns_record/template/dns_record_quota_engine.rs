use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the dns record quota calculation.
pub trait DnsRecordQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDnsRecordQuotaAdjuster {
    pub percent: f64,
}

impl DnsRecordQuotaAdjuster for PercentDnsRecordQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DnsRecordQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordQuotaAdjuster {
    pub minimum: f64,
}

impl DnsRecordQuotaAdjuster for MinimumDnsRecordQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordQuotaAdjuster {
    pub cap: f64,
}

impl DnsRecordQuotaAdjuster for CapDnsRecordQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the dns record quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordQuotaEngine {
    pub steps: Vec<Rc<dyn DnsRecordQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordQuotaEngine {
    pub fn evaluate(&mut self, item: &DnsRecordQuota) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordQuotaAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_dns_record_quota_engine() -> DnsRecordQuotaEngine {
    DnsRecordQuotaEngine {
        steps: vec![
            Rc::new(PercentDnsRecordQuotaAdjuster { percent: -13.0 }) as Rc<dyn DnsRecordQuotaAdjuster>,
            Rc::new(MinimumDnsRecordQuotaAdjuster { minimum: 26.7 }),
            Rc::new(CapDnsRecordQuotaAdjuster { cap: 10700.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 47615.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_quota_engine();
        let item = new_dns_record_quota("code-933", "title-573");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
