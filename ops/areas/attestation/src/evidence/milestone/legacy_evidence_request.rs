use super::*;

use acme_kit::text;

/// Where a legacy evidence request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyEvidenceRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyEvidenceRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyEvidenceRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy evidence request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceRequest {
    pub code: String,
    pub title: String,
    pub source: String,
    pub capacity: i64,
    pub minimum: f64,
    pub approved: bool,
    pub description: String,
    pub scope: String,
    pub status: LegacyEvidenceRequestStatus,
}

impl LegacyEvidenceRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyEvidenceRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyEvidenceRequestStatus) -> bool {
        match self.status {
            LegacyEvidenceRequestStatus::Requested => {
                match next {
                    LegacyEvidenceRequestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEvidenceRequestStatus::Granted => {
                match next {
                    LegacyEvidenceRequestStatus::Revoked | LegacyEvidenceRequestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEvidenceRequestStatus::Revoked => {
                return next == LegacyEvidenceRequestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyEvidenceRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy evidence request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.minimum + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyEvidenceRequest {
        LegacyEvidenceRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            capacity: self.capacity,
            minimum: self.minimum,
            approved: self.approved,
            description: self.description.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a legacy evidence request in status requested.
pub fn new_legacy_evidence_request(code: &str, title: &str) -> LegacyEvidenceRequest {
    LegacyEvidenceRequest {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        capacity: 0,
        minimum: 0.0,
        approved: false,
        description: "".to_string(),
        scope: "".to_string(),
        status: LegacyEvidenceRequestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_legacy_evidence_request("code-857", "title-544");
        assert_eq!(item.status, LegacyEvidenceRequestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_legacy_evidence_request("code-857", "title-544");
        if item.move_to(LegacyEvidenceRequestStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyEvidenceRequestStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_legacy_evidence_request("code-857", "title-544");
        item.status = LegacyEvidenceRequestStatus::Expired;
        assert!(
            item.move_to(LegacyEvidenceRequestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
