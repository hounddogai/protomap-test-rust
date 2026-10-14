use super::*;

use acme_kit::text;

/// Where a evidence manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceManifestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EvidenceManifestStatus {
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

impl std::fmt::Display for EvidenceManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceManifest {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub contact_email: String,
    pub ratio: f64,
    pub status: EvidenceManifestStatus,
}

impl EvidenceManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceManifestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceManifestStatus) -> bool {
        match self.status {
            EvidenceManifestStatus::Draft => {
                return next == EvidenceManifestStatus::Active || next == EvidenceManifestStatus::Closed;
            }
            EvidenceManifestStatus::Pending => {
                match next {
                    EvidenceManifestStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceManifestStatus::Active => {
                match next {
                    EvidenceManifestStatus::Suspended | EvidenceManifestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceManifestStatus::Suspended => {
                match next {
                    EvidenceManifestStatus::Closed => {
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

    pub fn move_to(&mut self, next: EvidenceManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.ratio * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceManifest {
        EvidenceManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            contact_email: self.contact_email.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a evidence manifest in status draft.
pub fn new_evidence_manifest(code: &str, label: &str) -> EvidenceManifest {
    EvidenceManifest {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        contact_email: "".to_string(),
        ratio: 0.0,
        status: EvidenceManifestStatus::Draft,
    }
}
