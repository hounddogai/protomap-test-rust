use super::*;

use acme_kit::text;

/// Where a dashboard rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DashboardRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DashboardRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard rollup as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRollup {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub currency: String,
    pub score: f64,
    pub expires_at: i64,
    pub issuer: String,
    pub ends_at: i64,
    pub external_ref: String,
    pub budget: f64,
    pub status: DashboardRollupStatus,
}

impl DashboardRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardRollupStatus) -> bool {
        match self.status {
            DashboardRollupStatus::Proposed => {
                match next {
                    DashboardRollupStatus::Approved | DashboardRollupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardRollupStatus::Approved => {
                match next {
                    DashboardRollupStatus::Rejected | DashboardRollupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardRollupStatus::Rejected => {
                return next == DashboardRollupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "dashboard rollup", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.budget - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardRollup {
        DashboardRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            currency: self.currency.clone(),
            score: self.score,
            expires_at: self.expires_at,
            issuer: self.issuer.clone(),
            ends_at: self.ends_at,
            external_ref: self.external_ref.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dashboard rollup in status proposed.
pub fn new_dashboard_rollup(code: &str, label: &str) -> DashboardRollup {
    DashboardRollup {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        currency: "".to_string(),
        score: 0.0,
        expires_at: 0,
        issuer: "".to_string(),
        ends_at: 0,
        external_ref: "".to_string(),
        budget: 0.0,
        status: DashboardRollupStatus::Proposed,
    }
}
