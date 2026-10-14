use super::*;

use acme_kit::text;

/// Where a primary data export profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryDataExportProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryDataExportProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryDataExportProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary data export profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDataExportProfile {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub namespace: String,
    pub host: String,
    pub threshold: f64,
    pub limit: i64,
    pub starts_at: i64,
    pub status: PrimaryDataExportProfileStatus,
}

impl PrimaryDataExportProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryDataExportProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryDataExportProfileStatus) -> bool {
        match self.status {
            PrimaryDataExportProfileStatus::Requested => {
                match next {
                    PrimaryDataExportProfileStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDataExportProfileStatus::Granted => {
                match next {
                    PrimaryDataExportProfileStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDataExportProfileStatus::Revoked => {
                match next {
                    PrimaryDataExportProfileStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryDataExportProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary data export profile {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.limit as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryDataExportProfile {
        PrimaryDataExportProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            namespace: self.namespace.clone(),
            host: self.host.clone(),
            threshold: self.threshold,
            limit: self.limit,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a primary data export profile in status requested.
pub fn new_primary_data_export_profile(code: &str, title: &str) -> PrimaryDataExportProfile {
    PrimaryDataExportProfile {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        namespace: "".to_string(),
        host: "".to_string(),
        threshold: 0.0,
        limit: 0,
        starts_at: 0,
        status: PrimaryDataExportProfileStatus::Requested,
    }
}
