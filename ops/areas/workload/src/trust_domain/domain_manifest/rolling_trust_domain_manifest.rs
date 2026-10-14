use super::*;

use acme_kit::text;

/// Where a rolling trust domain manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTrustDomainManifestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingTrustDomainManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingTrustDomainManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling trust domain manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTrustDomainManifest {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub title: String,
    pub threshold: f64,
    pub maximum: f64,
    pub status: RollingTrustDomainManifestStatus,
}

impl RollingTrustDomainManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTrustDomainManifestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTrustDomainManifestStatus) -> bool {
        match self.status {
            RollingTrustDomainManifestStatus::Requested => {
                match next {
                    RollingTrustDomainManifestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTrustDomainManifestStatus::Granted => {
                match next {
                    RollingTrustDomainManifestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTrustDomainManifestStatus::Revoked => {
                match next {
                    RollingTrustDomainManifestStatus::Expired => {
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

    pub fn move_to(&mut self, next: RollingTrustDomainManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling trust domain manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling trust domain manifest", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.threshold - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingTrustDomainManifest {
        RollingTrustDomainManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            title: self.title.clone(),
            threshold: self.threshold,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling trust domain manifest in status requested.
pub fn new_rolling_trust_domain_manifest(code: &str, name: &str) -> RollingTrustDomainManifest {
    RollingTrustDomainManifest {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        title: "".to_string(),
        threshold: 0.0,
        maximum: 0.0,
        status: RollingTrustDomainManifestStatus::Requested,
    }
}
