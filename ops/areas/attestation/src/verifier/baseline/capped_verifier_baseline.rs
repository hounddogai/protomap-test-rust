use super::*;

use acme_kit::text;

/// Where a capped verifier baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedVerifierBaselineStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedVerifierBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedVerifierBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped verifier baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierBaseline {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub floor_value: f64,
    pub audience: String,
    pub weight: f64,
    pub reviewer: String,
    pub starts_at: i64,
    pub archived: bool,
    pub fingerprint: String,
    pub owner_id: String,
    pub status: CappedVerifierBaselineStatus,
}

impl CappedVerifierBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedVerifierBaselineStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedVerifierBaselineStatus) -> bool {
        match self.status {
            CappedVerifierBaselineStatus::Requested => {
                match next {
                    CappedVerifierBaselineStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedVerifierBaselineStatus::Granted => {
                match next {
                    CappedVerifierBaselineStatus::Revoked | CappedVerifierBaselineStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedVerifierBaselineStatus::Revoked => {
                return next == CappedVerifierBaselineStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedVerifierBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped verifier baseline {} cannot move from {} to {}",
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

    pub fn net_volume(&self) -> f64 {
        ((self.amount - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedVerifierBaseline {
        CappedVerifierBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            floor_value: self.floor_value,
            audience: self.audience.clone(),
            weight: self.weight,
            reviewer: self.reviewer.clone(),
            starts_at: self.starts_at,
            archived: self.archived,
            fingerprint: self.fingerprint.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped verifier baseline in status requested.
pub fn new_capped_verifier_baseline(code: &str, title: &str) -> CappedVerifierBaseline {
    CappedVerifierBaseline {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        floor_value: 0.0,
        audience: "".to_string(),
        weight: 0.0,
        reviewer: "".to_string(),
        starts_at: 0,
        archived: false,
        fingerprint: "".to_string(),
        owner_id: "".to_string(),
        status: CappedVerifierBaselineStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_verifier_baseline("code-371", "title-369");
        assert_eq!(item.status, CappedVerifierBaselineStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_capped_verifier_baseline("code-371", "title-369");
        if item.move_to(CappedVerifierBaselineStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedVerifierBaselineStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_verifier_baseline("code-371", "title-369");
        item.status = CappedVerifierBaselineStatus::Expired;
        assert!(
            item.move_to(CappedVerifierBaselineStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
