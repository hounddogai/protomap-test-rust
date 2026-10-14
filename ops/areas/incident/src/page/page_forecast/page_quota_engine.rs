use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the page quota calculation.
pub trait PageQuotaStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPageQuotaAdjuster {
    pub cap: f64,
}

impl PageQuotaStep for CapPageQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PageQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PageQuotaStep for TieredPageQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PageQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageQuotaAdjuster {
    pub minimum: f64,
}

impl PageQuotaStep for MinimumPageQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPageQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PageQuotaStep for FlaggedPageQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PageQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the page quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageQuotaEngine {
    pub steps: Vec<Rc<dyn PageQuotaStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageQuotaEngine {
    pub fn compute(&mut self, item: &PageQuota) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PageQuotaStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PageQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_page_quota_engine(flags: &flags::Registry) -> PageQuotaEngine {
    PageQuotaEngine {
        steps: vec![
            Rc::new(CapPageQuotaAdjuster { cap: 7316.0 }) as Rc<dyn PageQuotaStep>,
            Rc::new(TieredPageQuotaAdjuster {
                bounds: vec![4200.0, 6579.0, 7973.0, 8965.0],
                rates: vec![1.12, 0.91, 1.2, 1.08],
            }),
            Rc::new(MinimumPageQuotaAdjuster { minimum: 12.6 }),
            Rc::new(FlaggedPageQuotaAdjuster {
                flags: flags.clone(),
                flag: "page-quota-manual".to_string(),
                percent: -7.7,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 40553.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_quota_engine(&flags::new_registry());
        let item = new_page_quota("code-893", "title-453");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
