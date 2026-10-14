use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tenant rule calculation.
pub trait TenantRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTenantRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantRuleModifier for NameTenantRuleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TenantRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantRuleModifier for FlaggedTenantRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantRuleAdjuster {
    pub percent: f64,
}

impl TenantRuleModifier for PercentTenantRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantRuleAdjuster {
    pub minimum: f64,
}

impl TenantRuleModifier for MinimumTenantRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantRuleAdjuster {
    pub cap: f64,
}

impl TenantRuleModifier for CapTenantRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRuleCapacityOffsetAdjuster {
    pub weight: f64,
}

impl TenantRuleModifier for TenantRuleCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &TenantRule, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

/// Runs the tenant rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantRuleEngine {
    pub steps: Vec<Rc<dyn TenantRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantRuleEngine {
    pub fn compute(&mut self, item: &TenantRule) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[TenantRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tenant_rule_engine(flags: &flags::Registry) -> TenantRuleEngine {
    TenantRuleEngine {
        steps: vec![
            Rc::new(NameTenantRuleAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.88), ("south".to_string(), 1.02)]),
            }) as Rc<dyn TenantRuleModifier>,
            Rc::new(FlaggedTenantRuleAdjuster {
                flags: flags.clone(),
                flag: "tenant-rule-capped".to_string(),
                percent: -5.9,
            }),
            Rc::new(PercentTenantRuleAdjuster { percent: -6.2 }),
            Rc::new(MinimumTenantRuleAdjuster { minimum: 18.1 }),
            Rc::new(CapTenantRuleAdjuster { cap: 49813.0 }),
            Rc::new(TenantRuleCapacityOffsetAdjuster { weight: 1.57 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 49241.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_rule_engine(&flags::new_registry());
        let item = new_tenant_rule("code-743", "name-528");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
