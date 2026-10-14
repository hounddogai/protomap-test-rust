use super::*;

use acme_kit::text;

/// Where a seasonal evidence score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalEvidenceScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalEvidenceScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalEvidenceScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal evidence score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEvidenceScore {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub minimum: f64,
    pub cipher: String,
    pub expires_at: i64,
    pub contact_email: String,
    pub quantity: i64,
    pub burst: i64,
    pub window_days: i64,
    pub status: SeasonalEvidenceScoreStatus,
}

impl SeasonalEvidenceScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalEvidenceScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalEvidenceScoreStatus) -> bool {
        match self.status {
            SeasonalEvidenceScoreStatus::Requested => {
                return next == SeasonalEvidenceScoreStatus::Revoked;
            }
            SeasonalEvidenceScoreStatus::Granted => {
                match next {
                    SeasonalEvidenceScoreStatus::Revoked | SeasonalEvidenceScoreStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalEvidenceScoreStatus::Revoked => {
                return next == SeasonalEvidenceScoreStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalEvidenceScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal evidence score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.expires_at as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalEvidenceScore {
        SeasonalEvidenceScore {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            minimum: self.minimum,
            cipher: self.cipher.clone(),
            expires_at: self.expires_at,
            contact_email: self.contact_email.clone(),
            quantity: self.quantity,
            burst: self.burst,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal evidence score in status requested.
pub fn new_seasonal_evidence_score(code: &str, name: &str) -> SeasonalEvidenceScore {
    SeasonalEvidenceScore {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        minimum: 0.0,
        cipher: "".to_string(),
        expires_at: 0,
        contact_email: "".to_string(),
        quantity: 0,
        burst: 0,
        window_days: 0,
        status: SeasonalEvidenceScoreStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_evidence_score("code-267", "name-697");
        assert_eq!(item.status, SeasonalEvidenceScoreStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_seasonal_evidence_score("code-267", "name-697");
        if item.move_to(SeasonalEvidenceScoreStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalEvidenceScoreStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_evidence_score("code-267", "name-697");
        item.status = SeasonalEvidenceScoreStatus::Expired;
        assert!(
            item.move_to(SeasonalEvidenceScoreStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
