use super::*;

use acme_kit::text;

/// Where a legacy probe review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyProbeReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyProbeReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyProbeReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy probe review as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeReview {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub cipher: String,
    pub phone: String,
    pub amount: f64,
    pub pinned: bool,
    pub status: LegacyProbeReviewStatus,
}

impl LegacyProbeReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyProbeReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyProbeReviewStatus) -> bool {
        match self.status {
            LegacyProbeReviewStatus::Requested => {
                return next == LegacyProbeReviewStatus::Granted;
            }
            LegacyProbeReviewStatus::Granted => {
                return next == LegacyProbeReviewStatus::Revoked || next == LegacyProbeReviewStatus::Expired;
            }
            LegacyProbeReviewStatus::Revoked => {
                match next {
                    LegacyProbeReviewStatus::Expired => {
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

    pub fn move_to(&mut self, next: LegacyProbeReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy probe review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.version as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyProbeReview {
        LegacyProbeReview {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            cipher: self.cipher.clone(),
            phone: self.phone.clone(),
            amount: self.amount,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy probe review in status requested.
pub fn new_legacy_probe_review(code: &str, name: &str) -> LegacyProbeReview {
    LegacyProbeReview {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        cipher: "".to_string(),
        phone: "".to_string(),
        amount: 0.0,
        pinned: false,
        status: LegacyProbeReviewStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_legacy_probe_review("code-650", "name-491");
        assert_eq!(item.status, LegacyProbeReviewStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_legacy_probe_review("code-650", "name-491");
        if item.move_to(LegacyProbeReviewStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyProbeReviewStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_legacy_probe_review("code-650", "name-491");
        item.status = LegacyProbeReviewStatus::Expired;
        assert!(
            item.move_to(LegacyProbeReviewStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
