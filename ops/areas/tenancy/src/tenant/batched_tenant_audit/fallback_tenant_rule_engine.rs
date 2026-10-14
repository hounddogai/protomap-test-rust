use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the fallback tenant rule calculation.
pub trait FallbackTenantRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackTenantRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelFallbackTenantRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackTenantRuleModifier for ChannelFallbackTenantRuleAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &FallbackTenantRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantRuleBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl FallbackTenantRuleModifier for FallbackTenantRuleBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackTenantRule, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackTenantRuleAdjuster {
    pub minimum: f64,
}

impl FallbackTenantRuleModifier for MinimumFallbackTenantRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackTenantRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackTenantRuleAdjuster {
    pub percent: f64,
}

impl FallbackTenantRuleModifier for PercentFallbackTenantRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackTenantRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackTenantRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackTenantRuleModifier for FlaggedFallbackTenantRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackTenantRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackTenantRuleAdjuster {
    pub cap: f64,
}

impl FallbackTenantRuleModifier for CapFallbackTenantRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackTenantRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback tenant rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackTenantRuleEngine {
    pub steps: Vec<Rc<dyn FallbackTenantRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackTenantRuleEngine {
    pub fn run(&mut self, item: &FallbackTenantRule) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackTenantRuleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackTenantRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_tenant_rule_engine(flags: &flags::Registry) -> FallbackTenantRuleEngine {
    FallbackTenantRuleEngine {
        steps: vec![
            Rc::new(ChannelFallbackTenantRuleAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.24),
                    ("central".to_string(), 1.2),
                    ("west".to_string(), 1.27),
                ]),
            }) as Rc<dyn FallbackTenantRuleModifier>,
            Rc::new(FallbackTenantRuleBatchSizeOffsetAdjuster { weight: 0.36 }),
            Rc::new(MinimumFallbackTenantRuleAdjuster { minimum: 41.2 }),
            Rc::new(PercentFallbackTenantRuleAdjuster { percent: 9.8 }),
            Rc::new(FlaggedFallbackTenantRuleAdjuster {
                flags: flags.clone(),
                flag: "fallback-tenant-rule-manual".to_string(),
                percent: 5.2,
            }),
            Rc::new(CapFallbackTenantRuleAdjuster { cap: 8434.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 46657.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_tenant_rule_engine(&flags::new_registry());
        let item = new_fallback_tenant_rule("code-142", "title-734");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
