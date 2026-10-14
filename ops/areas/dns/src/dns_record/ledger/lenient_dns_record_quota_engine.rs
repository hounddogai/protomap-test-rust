use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient dns record quota calculation.
pub trait LenientDnsRecordQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientDnsRecordQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordQuotaQuantityOffsetAdjuster {
    pub weight: f64,
}

impl LenientDnsRecordQuotaAdjuster for LenientDnsRecordQuotaQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &LenientDnsRecordQuota, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientDnsRecordQuotaAdjuster {
    pub minimum: f64,
}

impl LenientDnsRecordQuotaAdjuster for MinimumLenientDnsRecordQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientDnsRecordQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLenientDnsRecordQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientDnsRecordQuotaAdjuster for NameLenientDnsRecordQuotaAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LenientDnsRecordQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientDnsRecordQuotaAdjuster {
    pub cap: f64,
}

impl LenientDnsRecordQuotaAdjuster for CapLenientDnsRecordQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientDnsRecordQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientDnsRecordQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientDnsRecordQuotaAdjuster for FlaggedLenientDnsRecordQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientDnsRecordQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient dns record quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientDnsRecordQuotaEngine {
    pub steps: Vec<Rc<dyn LenientDnsRecordQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientDnsRecordQuotaEngine {
    pub fn run(&mut self, item: &LenientDnsRecordQuota) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientDnsRecordQuotaAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientDnsRecordQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_dns_record_quota_engine(flags: &flags::Registry) -> LenientDnsRecordQuotaEngine {
    LenientDnsRecordQuotaEngine {
        steps: vec![
            Rc::new(LenientDnsRecordQuotaQuantityOffsetAdjuster { weight: 0.13 })
                as Rc<dyn LenientDnsRecordQuotaAdjuster>,
            Rc::new(MinimumLenientDnsRecordQuotaAdjuster { minimum: 0.2 }),
            Rc::new(NameLenientDnsRecordQuotaAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.14), ("west".to_string(), 1.15)]),
            }),
            Rc::new(CapLenientDnsRecordQuotaAdjuster { cap: 26377.0 }),
            Rc::new(FlaggedLenientDnsRecordQuotaAdjuster {
                flags: flags.clone(),
                flag: "lenient-dns-record-quota-manual".to_string(),
                percent: 9.6,
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 10523.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_dns_record_quota_engine(&flags::new_registry());
        let item = new_lenient_dns_record_quota("code-966", "name-818");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
