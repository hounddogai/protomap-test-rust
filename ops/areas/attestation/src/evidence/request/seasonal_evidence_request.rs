use super::*;

use acme_kit::text;

/// Where a seasonal evidence request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalEvidenceRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalEvidenceRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalEvidenceRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal evidence request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEvidenceRequest {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub cipher: String,
    pub updated_at: i64,
    pub starts_at: i64,
    pub segment: String,
    pub ratio: f64,
    pub status: SeasonalEvidenceRequestStatus,
}

impl SeasonalEvidenceRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalEvidenceRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalEvidenceRequestStatus) -> bool {
        match self.status {
            SeasonalEvidenceRequestStatus::Requested => {
                match next {
                    SeasonalEvidenceRequestStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalEvidenceRequestStatus::Granted => {
                match next {
                    SeasonalEvidenceRequestStatus::Revoked | SeasonalEvidenceRequestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalEvidenceRequestStatus::Revoked => {
                return next == SeasonalEvidenceRequestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalEvidenceRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal evidence request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal evidence request", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.ratio * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalEvidenceRequest {
        SeasonalEvidenceRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            cipher: self.cipher.clone(),
            updated_at: self.updated_at,
            starts_at: self.starts_at,
            segment: self.segment.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a seasonal evidence request in status requested.
pub fn new_seasonal_evidence_request(code: &str, name: &str) -> SeasonalEvidenceRequest {
    SeasonalEvidenceRequest {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        cipher: "".to_string(),
        updated_at: 0,
        starts_at: 0,
        segment: "".to_string(),
        ratio: 0.0,
        status: SeasonalEvidenceRequestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_evidence_request("code-499", "name-835");
        assert_eq!(item.status, SeasonalEvidenceRequestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_seasonal_evidence_request("code-499", "name-835");
        if item.move_to(SeasonalEvidenceRequestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalEvidenceRequestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_evidence_request("code-499", "name-835");
        item.status = SeasonalEvidenceRequestStatus::Expired;
        assert!(
            item.move_to(SeasonalEvidenceRequestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
