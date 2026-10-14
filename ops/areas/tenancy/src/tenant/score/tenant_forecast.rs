use super::*;

use acme_kit::text;

/// Where a tenant forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TenantForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for TenantForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tenant forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantForecast {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub rotation_days: i64,
    pub grace_days: i64,
    pub quantity: i64,
    pub host: String,
    pub duration_minutes: i64,
    pub status: TenantForecastStatus,
}

impl TenantForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantForecastStatus) -> bool {
        match self.status {
            TenantForecastStatus::Open => {
                return next == TenantForecastStatus::InReview || next == TenantForecastStatus::Resolved;
            }
            TenantForecastStatus::InReview => {
                return next == TenantForecastStatus::Resolved;
            }
            TenantForecastStatus::Resolved => {
                match next {
                    TenantForecastStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantForecastStatus::Reopened => {
                return next == TenantForecastStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.quantity as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantForecast {
        TenantForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            rotation_days: self.rotation_days,
            grace_days: self.grace_days,
            quantity: self.quantity,
            host: self.host.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tenant forecast in status open.
pub fn new_tenant_forecast(code: &str, label: &str) -> TenantForecast {
    TenantForecast {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        rotation_days: 0,
        grace_days: 0,
        quantity: 0,
        host: "".to_string(),
        duration_minutes: 0,
        status: TenantForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tenant_forecast("code-497", "label-458");
        assert_eq!(item.status, TenantForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_tenant_forecast("code-497", "label-458");
        if item.move_to(TenantForecastStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantForecastStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tenant_forecast("code-497", "label-458");
        item.status = TenantForecastStatus::Archived;
        assert!(
            item.move_to(TenantForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
