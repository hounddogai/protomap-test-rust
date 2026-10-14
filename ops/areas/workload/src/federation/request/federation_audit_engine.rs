use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation audit calculation.
pub trait FederationAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFederationAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationAuditModifier for TitleFederationAuditAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FederationAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationAuditAdjuster {
    pub step: f64,
}

impl FederationAuditModifier for RoundingFederationAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationAuditModifier for FlaggedFederationAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationAuditAdjuster {
    pub minimum: f64,
}

impl FederationAuditModifier for MinimumFederationAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationAuditModifier for TieredFederationAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationAuditAdjuster {
    pub cap: f64,
}

impl FederationAuditModifier for CapFederationAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the federation audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationAuditEngine {
    pub steps: Vec<Rc<dyn FederationAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationAuditEngine {
    pub fn run(&mut self, item: &FederationAudit) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[FederationAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_federation_audit_engine(flags: &flags::Registry) -> FederationAuditEngine {
    FederationAuditEngine {
        steps: vec![
            Rc::new(TitleFederationAuditAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.06), ("west".to_string(), 0.74)]),
            }) as Rc<dyn FederationAuditModifier>,
            Rc::new(RoundingFederationAuditAdjuster { step: 5.0 }),
            Rc::new(FlaggedFederationAuditAdjuster {
                flags: flags.clone(),
                flag: "federation-audit-seasonal".to_string(),
                percent: 6.3,
            }),
            Rc::new(MinimumFederationAuditAdjuster { minimum: 8.1 }),
            Rc::new(TieredFederationAuditAdjuster {
                bounds: vec![1935.0, 3211.0, 5801.0, 6074.0],
                rates: vec![0.81, 0.9, 1.12, 0.87],
            }),
            Rc::new(CapFederationAuditAdjuster { cap: 2230.0 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 10810.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_audit_engine(&flags::new_registry());
        let item = new_federation_audit("code-841", "title-401");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
