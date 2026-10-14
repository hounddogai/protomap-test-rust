use super::*;

use acme_kit::text;

/// Where a verifier estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierEstimateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl VerifierEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for VerifierEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierEstimate {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub timezone: String,
    pub notes: String,
    pub port: i64,
    pub capacity: i64,
    pub status: VerifierEstimateStatus,
}

impl VerifierEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierEstimateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierEstimateStatus) -> bool {
        match self.status {
            VerifierEstimateStatus::Draft => {
                return next == VerifierEstimateStatus::Active || next == VerifierEstimateStatus::Suspended;
            }
            VerifierEstimateStatus::Pending => {
                return next == VerifierEstimateStatus::Closed;
            }
            VerifierEstimateStatus::Active => {
                return next == VerifierEstimateStatus::Suspended || next == VerifierEstimateStatus::Closed;
            }
            VerifierEstimateStatus::Suspended => {
                match next {
                    VerifierEstimateStatus::Closed => {
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

    pub fn move_to(&mut self, next: VerifierEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "verifier estimate", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.capacity as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierEstimate {
        VerifierEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            timezone: self.timezone.clone(),
            notes: self.notes.clone(),
            port: self.port,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a verifier estimate in status draft.
pub fn new_verifier_estimate(code: &str, name: &str) -> VerifierEstimate {
    VerifierEstimate {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        timezone: "".to_string(),
        notes: "".to_string(),
        port: 0,
        capacity: 0,
        status: VerifierEstimateStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_verifier_estimate("code-609", "name-576");
        assert_eq!(item.status, VerifierEstimateStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_verifier_estimate("code-609", "name-576");
        if item.move_to(VerifierEstimateStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierEstimateStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_verifier_estimate("code-609", "name-576");
        item.status = VerifierEstimateStatus::Closed;
        assert!(
            item.move_to(VerifierEstimateStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
