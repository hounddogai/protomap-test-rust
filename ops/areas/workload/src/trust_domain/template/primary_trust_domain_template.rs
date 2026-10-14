use super::*;

use acme_kit::text;

/// Where a primary trust domain template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTrustDomainTemplateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryTrustDomainTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryTrustDomainTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary trust domain template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainTemplate {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub rotation_days: i64,
    pub active: bool,
    pub reviewer: String,
    pub expires_at: i64,
    pub margin: f64,
    pub status: PrimaryTrustDomainTemplateStatus,
}

impl PrimaryTrustDomainTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTrustDomainTemplateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTrustDomainTemplateStatus) -> bool {
        match self.status {
            PrimaryTrustDomainTemplateStatus::Created => {
                match next {
                    PrimaryTrustDomainTemplateStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTrustDomainTemplateStatus::Validated => {
                return next == PrimaryTrustDomainTemplateStatus::Published
                    || next == PrimaryTrustDomainTemplateStatus::Retired;
            }
            PrimaryTrustDomainTemplateStatus::Published => {
                return next == PrimaryTrustDomainTemplateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryTrustDomainTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary trust domain template {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.margin - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryTrustDomainTemplate {
        PrimaryTrustDomainTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            rotation_days: self.rotation_days,
            active: self.active,
            reviewer: self.reviewer.clone(),
            expires_at: self.expires_at,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary trust domain template in status created.
pub fn new_primary_trust_domain_template(code: &str, title: &str) -> PrimaryTrustDomainTemplate {
    PrimaryTrustDomainTemplate {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        rotation_days: 0,
        active: false,
        reviewer: "".to_string(),
        expires_at: 0,
        margin: 0.0,
        status: PrimaryTrustDomainTemplateStatus::Created,
    }
}
