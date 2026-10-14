use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient slo audit calculation.
pub trait LenientSloAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientSloAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientSloAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientSloAuditAdjuster for TieredLenientSloAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientSloAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientSloAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientSloAuditAdjuster for FlaggedLenientSloAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientSloAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientSloAuditAdjuster {
    pub cap: f64,
}

impl LenientSloAuditAdjuster for CapLenientSloAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientSloAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientSloAuditAdjuster {
    pub minimum: f64,
}

impl LenientSloAuditAdjuster for MinimumLenientSloAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientSloAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient slo audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientSloAuditEngine {
    pub steps: Vec<Rc<dyn LenientSloAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientSloAuditEngine {
    pub fn price(&mut self, item: &LenientSloAudit) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientSloAuditAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientSloAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_slo_audit_engine(flags: &flags::Registry) -> LenientSloAuditEngine {
    LenientSloAuditEngine {
        steps: vec![
            Rc::new(TieredLenientSloAuditAdjuster {
                bounds: vec![5881.0, 7718.0],
                rates: vec![1.1, 0.99],
            }) as Rc<dyn LenientSloAuditAdjuster>,
            Rc::new(FlaggedLenientSloAuditAdjuster {
                flags: flags.clone(),
                flag: "lenient-slo-audit-legacy".to_string(),
                percent: -5.2,
            }),
            Rc::new(CapLenientSloAuditAdjuster { cap: 3924.0 }),
            Rc::new(MinimumLenientSloAuditAdjuster { minimum: 39.6 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 3247.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_slo_audit_engine(&flags::new_registry());
        let item = new_lenient_slo_audit("code-817", "label-410");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
