use super::*;

use acme_kit::text;

/// Where a legacy rule metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRuleMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyRuleMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyRuleMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy rule metric as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRuleMetric {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub notes: String,
    pub phone: String,
    pub status: LegacyRuleMetricStatus,
}

impl LegacyRuleMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRuleMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRuleMetricStatus) -> bool {
        match self.status {
            LegacyRuleMetricStatus::Created => {
                return next == LegacyRuleMetricStatus::Validated || next == LegacyRuleMetricStatus::Retired;
            }
            LegacyRuleMetricStatus::Validated => {
                match next {
                    LegacyRuleMetricStatus::Published | LegacyRuleMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRuleMetricStatus::Published => {
                match next {
                    LegacyRuleMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyRuleMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy rule metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> LegacyRuleMetric {
        LegacyRuleMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            notes: self.notes.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy rule metric in status created.
pub fn new_legacy_rule_metric(code: &str, label: &str) -> LegacyRuleMetric {
    LegacyRuleMetric {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        notes: "".to_string(),
        phone: "".to_string(),
        status: LegacyRuleMetricStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_rule_metric("code-15", "label-781");
        assert_eq!(item.status, LegacyRuleMetricStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_legacy_rule_metric("code-15", "label-781");
        if item.move_to(LegacyRuleMetricStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRuleMetricStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_rule_metric("code-15", "label-781");
        item.status = LegacyRuleMetricStatus::Retired;
        assert!(
            item.move_to(LegacyRuleMetricStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
