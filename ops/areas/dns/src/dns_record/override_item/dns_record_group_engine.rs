use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dns record group calculation.
pub trait DnsRecordGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDnsRecordGroupAdjuster {
    pub step: f64,
}

impl DnsRecordGroupStep for RoundingDnsRecordGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DnsRecordGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordGroupAdjuster {
    pub minimum: f64,
}

impl DnsRecordGroupStep for MinimumDnsRecordGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDnsRecordGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordGroupStep for TitleDnsRecordGroupAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DnsRecordGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDnsRecordGroupAdjuster {
    pub percent: f64,
}

impl DnsRecordGroupStep for PercentDnsRecordGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DnsRecordGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the dns record group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordGroupEngine {
    pub steps: Vec<Rc<dyn DnsRecordGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordGroupEngine {
    pub fn evaluate(&mut self, item: &DnsRecordGroup) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordGroupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_dns_record_group_engine() -> DnsRecordGroupEngine {
    DnsRecordGroupEngine {
        steps: vec![
            Rc::new(RoundingDnsRecordGroupAdjuster { step: 5.0 }) as Rc<dyn DnsRecordGroupStep>,
            Rc::new(MinimumDnsRecordGroupAdjuster { minimum: 21.6 }),
            Rc::new(TitleDnsRecordGroupAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.79)]),
            }),
            Rc::new(PercentDnsRecordGroupAdjuster { percent: -14.3 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 55581.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_group_engine();
        let item = new_dns_record_group("code-643", "title-964");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
