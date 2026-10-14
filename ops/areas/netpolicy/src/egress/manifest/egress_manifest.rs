use super::*;

use acme_kit::text;

/// Where a egress manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressManifestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EgressManifestStatus {
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

impl std::fmt::Display for EgressManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One egress manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressManifest {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub floor_value: f64,
    pub burst: i64,
    pub issuer: String,
    pub key_id: String,
    pub budget: f64,
    pub updated_at: i64,
    pub status: EgressManifestStatus,
}

impl EgressManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressManifestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressManifestStatus) -> bool {
        match self.status {
            EgressManifestStatus::Draft => {
                return next == EgressManifestStatus::Pending;
            }
            EgressManifestStatus::Pending => {
                return next == EgressManifestStatus::Active;
            }
            EgressManifestStatus::Active => {
                match next {
                    EgressManifestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressManifestStatus::Suspended => {
                return next == EgressManifestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress manifest {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.updated_at as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressManifest {
        EgressManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            floor_value: self.floor_value,
            burst: self.burst,
            issuer: self.issuer.clone(),
            key_id: self.key_id.clone(),
            budget: self.budget,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a egress manifest in status draft.
pub fn new_egress_manifest(code: &str, label: &str) -> EgressManifest {
    EgressManifest {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        floor_value: 0.0,
        burst: 0,
        issuer: "".to_string(),
        key_id: "".to_string(),
        budget: 0.0,
        updated_at: 0,
        status: EgressManifestStatus::Draft,
    }
}
