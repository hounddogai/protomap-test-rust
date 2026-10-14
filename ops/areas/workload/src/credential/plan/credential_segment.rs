use super::*;

use acme_kit::text;

/// Where a credential segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialSegmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CredentialSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CredentialSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSegment {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub threshold: f64,
    pub region: String,
    pub status: CredentialSegmentStatus,
}

impl CredentialSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialSegmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialSegmentStatus) -> bool {
        match self.status {
            CredentialSegmentStatus::Requested => {
                return next == CredentialSegmentStatus::Revoked;
            }
            CredentialSegmentStatus::Granted => {
                return next == CredentialSegmentStatus::Revoked || next == CredentialSegmentStatus::Expired;
            }
            CredentialSegmentStatus::Revoked => {
                return next == CredentialSegmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.duration_minutes as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialSegment {
        CredentialSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            threshold: self.threshold,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a credential segment in status requested.
pub fn new_credential_segment(code: &str, label: &str) -> CredentialSegment {
    CredentialSegment {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        threshold: 0.0,
        region: "".to_string(),
        status: CredentialSegmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_credential_segment("code-556", "label-108");
        assert_eq!(item.status, CredentialSegmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_credential_segment("code-556", "label-108");
        if item.move_to(CredentialSegmentStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialSegmentStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_credential_segment("code-556", "label-108");
        item.status = CredentialSegmentStatus::Expired;
        assert!(
            item.move_to(CredentialSegmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
