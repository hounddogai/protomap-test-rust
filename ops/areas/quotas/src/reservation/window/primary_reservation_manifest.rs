use super::*;

use acme_kit::text;

/// Where a primary reservation manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryReservationManifestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryReservationManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryReservationManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary reservation manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationManifest {
    pub code: String,
    pub title: String,
    pub source: String,
    pub ceiling: f64,
    pub starts_at: i64,
    pub active: bool,
    pub version: i64,
    pub threshold: f64,
    pub issuer: String,
    pub floor_value: f64,
    pub limit: i64,
    pub status: PrimaryReservationManifestStatus,
}

impl PrimaryReservationManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryReservationManifestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryReservationManifestStatus) -> bool {
        match self.status {
            PrimaryReservationManifestStatus::Requested => {
                return next == PrimaryReservationManifestStatus::Granted;
            }
            PrimaryReservationManifestStatus::Granted => {
                match next {
                    PrimaryReservationManifestStatus::Revoked | PrimaryReservationManifestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryReservationManifestStatus::Revoked => {
                match next {
                    PrimaryReservationManifestStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryReservationManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary reservation manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary reservation manifest", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ceiling + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryReservationManifest {
        PrimaryReservationManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            ceiling: self.ceiling,
            starts_at: self.starts_at,
            active: self.active,
            version: self.version,
            threshold: self.threshold,
            issuer: self.issuer.clone(),
            floor_value: self.floor_value,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary reservation manifest in status requested.
pub fn new_primary_reservation_manifest(code: &str, title: &str) -> PrimaryReservationManifest {
    PrimaryReservationManifest {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        ceiling: 0.0,
        starts_at: 0,
        active: false,
        version: 0,
        threshold: 0.0,
        issuer: "".to_string(),
        floor_value: 0.0,
        limit: 0,
        status: PrimaryReservationManifestStatus::Requested,
    }
}
