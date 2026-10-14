use super::*;

use acme_kit::text;

/// Where a slo template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SloTemplateStatus {
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

impl std::fmt::Display for SloTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo template as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloTemplate {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub key_id: String,
    pub quantity: i64,
    pub scope: String,
    pub contact_email: String,
    pub country: String,
    pub amount: f64,
    pub ceiling: f64,
    pub weight: f64,
    pub status: SloTemplateStatus,
}

impl SloTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloTemplateStatus) -> bool {
        match self.status {
            SloTemplateStatus::Open => {
                return next == SloTemplateStatus::Resolved || next == SloTemplateStatus::Archived;
            }
            SloTemplateStatus::InReview => {
                match next {
                    SloTemplateStatus::Resolved | SloTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloTemplateStatus::Resolved => {
                match next {
                    SloTemplateStatus::Reopened | SloTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloTemplateStatus::Reopened => {
                return next == SloTemplateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo template {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.weight - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloTemplate {
        SloTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            key_id: self.key_id.clone(),
            quantity: self.quantity,
            scope: self.scope.clone(),
            contact_email: self.contact_email.clone(),
            country: self.country.clone(),
            amount: self.amount,
            ceiling: self.ceiling,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a slo template in status open.
pub fn new_slo_template(code: &str, name: &str) -> SloTemplate {
    SloTemplate {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        key_id: "".to_string(),
        quantity: 0,
        scope: "".to_string(),
        contact_email: "".to_string(),
        country: "".to_string(),
        amount: 0.0,
        ceiling: 0.0,
        weight: 0.0,
        status: SloTemplateStatus::Open,
    }
}
