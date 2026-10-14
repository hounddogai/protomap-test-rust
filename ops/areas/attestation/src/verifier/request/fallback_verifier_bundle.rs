use super::*;

use acme_kit::text;

/// Where a fallback verifier bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackVerifierBundleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackVerifierBundleStatus {
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

impl std::fmt::Display for FallbackVerifierBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback verifier bundle as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierBundle {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub name: String,
    pub namespace: String,
    pub threshold: f64,
    pub starts_at: i64,
    pub rotation_days: i64,
    pub ends_at: i64,
    pub active: bool,
    pub status: FallbackVerifierBundleStatus,
}

impl FallbackVerifierBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackVerifierBundleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackVerifierBundleStatus) -> bool {
        match self.status {
            FallbackVerifierBundleStatus::Draft => {
                return next == FallbackVerifierBundleStatus::Active;
            }
            FallbackVerifierBundleStatus::Pending => {
                return next == FallbackVerifierBundleStatus::Closed;
            }
            FallbackVerifierBundleStatus::Active => {
                match next {
                    FallbackVerifierBundleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackVerifierBundleStatus::Suspended => {
                match next {
                    FallbackVerifierBundleStatus::Closed => {
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

    pub fn move_to(&mut self, next: FallbackVerifierBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback verifier bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.starts_at as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackVerifierBundle {
        FallbackVerifierBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            threshold: self.threshold,
            starts_at: self.starts_at,
            rotation_days: self.rotation_days,
            ends_at: self.ends_at,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback verifier bundle in status draft.
pub fn new_fallback_verifier_bundle(code: &str, title: &str) -> FallbackVerifierBundle {
    FallbackVerifierBundle {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        name: "".to_string(),
        namespace: "".to_string(),
        threshold: 0.0,
        starts_at: 0,
        rotation_days: 0,
        ends_at: 0,
        active: false,
        status: FallbackVerifierBundleStatus::Draft,
    }
}
