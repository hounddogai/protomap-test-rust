use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a manual forecast budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualForecastBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualForecastBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualForecastBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual forecast budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastBudget {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub attributes: BTreeMap<String, String>,
    pub summary: String,
    pub ends_at: i64,
    pub issuer: String,
    pub namespace: String,
    pub expires_at: i64,
    pub status: ManualForecastBudgetStatus,
}

impl ManualForecastBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualForecastBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualForecastBudgetStatus) -> bool {
        match self.status {
            ManualForecastBudgetStatus::Created => {
                return next == ManualForecastBudgetStatus::Published;
            }
            ManualForecastBudgetStatus::Validated => {
                match next {
                    ManualForecastBudgetStatus::Published | ManualForecastBudgetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualForecastBudgetStatus::Published => {
                return next == ManualForecastBudgetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualForecastBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual forecast budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.expires_at as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualForecastBudget {
        ManualForecastBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            attributes: self.attributes.clone(),
            summary: self.summary.clone(),
            ends_at: self.ends_at,
            issuer: self.issuer.clone(),
            namespace: self.namespace.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a manual forecast budget in status created.
pub fn new_manual_forecast_budget(code: &str, label: &str) -> ManualForecastBudget {
    ManualForecastBudget {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        attributes: BTreeMap::new(),
        summary: "".to_string(),
        ends_at: 0,
        issuer: "".to_string(),
        namespace: "".to_string(),
        expires_at: 0,
        status: ManualForecastBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_forecast_budget("code-883", "label-146");
        assert_eq!(item.status, ManualForecastBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_manual_forecast_budget("code-883", "label-146");
        if item.move_to(ManualForecastBudgetStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualForecastBudgetStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_forecast_budget("code-883", "label-146");
        item.status = ManualForecastBudgetStatus::Retired;
        assert!(
            item.move_to(ManualForecastBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
