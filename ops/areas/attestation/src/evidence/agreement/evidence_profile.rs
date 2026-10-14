use super::*;

use acme_kit::text;

/// Where a evidence profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl EvidenceProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for EvidenceProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceProfile {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub quantity: i64,
    pub threshold: f64,
    pub fingerprint: String,
    pub category: String,
    pub ends_at: i64,
    pub summary: String,
    pub country: String,
    pub status: EvidenceProfileStatus,
}

impl EvidenceProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceProfileStatus) -> bool {
        match self.status {
            EvidenceProfileStatus::Requested => {
                match next {
                    EvidenceProfileStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceProfileStatus::Granted => {
                match next {
                    EvidenceProfileStatus::Revoked | EvidenceProfileStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceProfileStatus::Revoked => {
                match next {
                    EvidenceProfileStatus::Expired => {
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

    pub fn move_to(&mut self, next: EvidenceProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence profile {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.quantity as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceProfile {
        EvidenceProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            quantity: self.quantity,
            threshold: self.threshold,
            fingerprint: self.fingerprint.clone(),
            category: self.category.clone(),
            ends_at: self.ends_at,
            summary: self.summary.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a evidence profile in status requested.
pub fn new_evidence_profile(code: &str, title: &str) -> EvidenceProfile {
    EvidenceProfile {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        quantity: 0,
        threshold: 0.0,
        fingerprint: "".to_string(),
        category: "".to_string(),
        ends_at: 0,
        summary: "".to_string(),
        country: "".to_string(),
        status: EvidenceProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_evidence_profile("code-328", "title-739");
        assert_eq!(item.status, EvidenceProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_evidence_profile("code-328", "title-739");
        if item.move_to(EvidenceProfileStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceProfileStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_evidence_profile("code-328", "title-739");
        item.status = EvidenceProfileStatus::Expired;
        assert!(
            item.move_to(EvidenceProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
