use super::*;

use acme_kit::text;

/// Where a primary dashboard template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryDashboardTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryDashboardTemplateStatus {
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

impl std::fmt::Display for PrimaryDashboardTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary dashboard template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardTemplate {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub quantity: i64,
    pub key_id: String,
    pub ttl_seconds: i64,
    pub name: String,
    pub issuer: String,
    pub grace_days: i64,
    pub status: PrimaryDashboardTemplateStatus,
}

impl PrimaryDashboardTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryDashboardTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryDashboardTemplateStatus) -> bool {
        match self.status {
            PrimaryDashboardTemplateStatus::Open => {
                match next {
                    PrimaryDashboardTemplateStatus::InReview | PrimaryDashboardTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDashboardTemplateStatus::InReview => {
                match next {
                    PrimaryDashboardTemplateStatus::Reopened | PrimaryDashboardTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDashboardTemplateStatus::Resolved => {
                return next == PrimaryDashboardTemplateStatus::Archived;
            }
            PrimaryDashboardTemplateStatus::Reopened => {
                return next == PrimaryDashboardTemplateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryDashboardTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary dashboard template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.quantity as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryDashboardTemplate {
        PrimaryDashboardTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            quantity: self.quantity,
            key_id: self.key_id.clone(),
            ttl_seconds: self.ttl_seconds,
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary dashboard template in status open.
pub fn new_primary_dashboard_template(code: &str, title: &str) -> PrimaryDashboardTemplate {
    PrimaryDashboardTemplate {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        quantity: 0,
        key_id: "".to_string(),
        ttl_seconds: 0,
        name: "".to_string(),
        issuer: "".to_string(),
        grace_days: 0,
        status: PrimaryDashboardTemplateStatus::Open,
    }
}
