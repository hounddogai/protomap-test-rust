use super::*;

use acme_kit::text;

/// Where a capped token claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedTokenClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedTokenClaimStatus {
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

impl std::fmt::Display for CappedTokenClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped token claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenClaim {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub cluster: String,
    pub phone: String,
    pub window_days: i64,
    pub created_at: i64,
    pub status: CappedTokenClaimStatus,
}

impl CappedTokenClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedTokenClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedTokenClaimStatus) -> bool {
        match self.status {
            CappedTokenClaimStatus::Planned => {
                return next == CappedTokenClaimStatus::Completed;
            }
            CappedTokenClaimStatus::Scheduled => {
                match next {
                    CappedTokenClaimStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedTokenClaimStatus::InProgress => {
                match next {
                    CappedTokenClaimStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedTokenClaimStatus::Completed => {
                return next == CappedTokenClaimStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedTokenClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped token claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.window_days as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedTokenClaim {
        CappedTokenClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            cluster: self.cluster.clone(),
            phone: self.phone.clone(),
            window_days: self.window_days,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a capped token claim in status planned.
pub fn new_capped_token_claim(code: &str, label: &str) -> CappedTokenClaim {
    CappedTokenClaim {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        cluster: "".to_string(),
        phone: "".to_string(),
        window_days: 0,
        created_at: 0,
        status: CappedTokenClaimStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_capped_token_claim("code-225", "label-609");
        assert_eq!(item.status, CappedTokenClaimStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_capped_token_claim("code-225", "label-609");
        if item.move_to(CappedTokenClaimStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedTokenClaimStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_capped_token_claim("code-225", "label-609");
        item.status = CappedTokenClaimStatus::Cancelled;
        assert!(
            item.move_to(CappedTokenClaimStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
