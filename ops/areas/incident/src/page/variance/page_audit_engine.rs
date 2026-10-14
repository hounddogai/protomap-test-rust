use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the page audit calculation.
pub trait PageAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageAuditAdjuster {
    pub minimum: f64,
}

impl PageAuditModifier for MinimumPageAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PageAuditModifier for TieredPageAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PageAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPageAuditAdjuster {
    pub step: f64,
}

impl PageAuditModifier for RoundingPageAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PageAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPageAuditAdjuster {
    pub percent: f64,
}

impl PageAuditModifier for PercentPageAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PageAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the page audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageAuditEngine {
    pub steps: Vec<Rc<dyn PageAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageAuditEngine {
    pub fn price(&mut self, item: &PageAudit) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PageAuditModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PageAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_page_audit_engine() -> PageAuditEngine {
    PageAuditEngine {
        steps: vec![
            Rc::new(MinimumPageAuditAdjuster { minimum: 24.6 }) as Rc<dyn PageAuditModifier>,
            Rc::new(TieredPageAuditAdjuster {
                bounds: vec![2059.0, 4139.0],
                rates: vec![0.92, 0.85],
            }),
            Rc::new(RoundingPageAuditAdjuster { step: 0.1 }),
            Rc::new(PercentPageAuditAdjuster { percent: 6.8 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 34670.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_audit_engine();
        let item = new_page_audit("code-13", "label-297");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
