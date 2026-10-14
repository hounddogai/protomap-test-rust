use super::*;

use acme_kit::text;

/// Where a default evidence quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultEvidenceQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultEvidenceQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for DefaultEvidenceQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default evidence quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceQuota {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub audience: String,
    pub threshold: f64,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub burst: i64,
    pub status: DefaultEvidenceQuotaStatus,
}

impl DefaultEvidenceQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultEvidenceQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultEvidenceQuotaStatus) -> bool {
        match self.status {
            DefaultEvidenceQuotaStatus::Draft => {
                match next {
                    DefaultEvidenceQuotaStatus::Active | DefaultEvidenceQuotaStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultEvidenceQuotaStatus::Pending => {
                match next {
                    DefaultEvidenceQuotaStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultEvidenceQuotaStatus::Active => {
                match next {
                    DefaultEvidenceQuotaStatus::Suspended | DefaultEvidenceQuotaStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultEvidenceQuotaStatus::Suspended => {
                return next == DefaultEvidenceQuotaStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultEvidenceQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default evidence quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.burst as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultEvidenceQuota {
        DefaultEvidenceQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            audience: self.audience.clone(),
            threshold: self.threshold,
            tags: self.tags.clone(),
            created_at: self.created_at,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default evidence quota in status draft.
pub fn new_default_evidence_quota(code: &str, title: &str) -> DefaultEvidenceQuota {
    DefaultEvidenceQuota {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        audience: "".to_string(),
        threshold: 0.0,
        tags: Vec::new(),
        created_at: 0,
        burst: 0,
        status: DefaultEvidenceQuotaStatus::Draft,
    }
}
