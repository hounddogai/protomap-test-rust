use super::*;

use acme_kit::text;

/// Where a trust domain manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TrustDomainManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TrustDomainManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain manifest as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainManifest {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub country: String,
    pub tenant_id: String,
    pub rotation_days: i64,
    pub cipher: String,
    pub summary: String,
    pub ratio: f64,
    pub quota_units: i64,
    pub reviewer: String,
    pub status: TrustDomainManifestStatus,
}

impl TrustDomainManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainManifestStatus) -> bool {
        match self.status {
            TrustDomainManifestStatus::Planned => {
                return next == TrustDomainManifestStatus::InProgress || next == TrustDomainManifestStatus::Completed;
            }
            TrustDomainManifestStatus::Scheduled => {
                match next {
                    TrustDomainManifestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainManifestStatus::InProgress => {
                return next == TrustDomainManifestStatus::Cancelled;
            }
            TrustDomainManifestStatus::Completed => {
                match next {
                    TrustDomainManifestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TrustDomainManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "trust domain manifest", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ratio - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainManifest {
        TrustDomainManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            country: self.country.clone(),
            tenant_id: self.tenant_id.clone(),
            rotation_days: self.rotation_days,
            cipher: self.cipher.clone(),
            summary: self.summary.clone(),
            ratio: self.ratio,
            quota_units: self.quota_units,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a trust domain manifest in status planned.
pub fn new_trust_domain_manifest(code: &str, label: &str) -> TrustDomainManifest {
    TrustDomainManifest {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        country: "".to_string(),
        tenant_id: "".to_string(),
        rotation_days: 0,
        cipher: "".to_string(),
        summary: "".to_string(),
        ratio: 0.0,
        quota_units: 0,
        reviewer: "".to_string(),
        status: TrustDomainManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_trust_domain_manifest("code-808", "label-836");
        assert_eq!(item.status, TrustDomainManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_trust_domain_manifest("code-808", "label-836");
        if item.move_to(TrustDomainManifestStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainManifestStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_trust_domain_manifest("code-808", "label-836");
        item.status = TrustDomainManifestStatus::Cancelled;
        assert!(
            item.move_to(TrustDomainManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
