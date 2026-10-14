use super::*;

use acme_kit::text;

/// Where a limit profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitProfileStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LimitProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LimitProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitProfile {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub amount: f64,
    pub notes: String,
    pub minimum: f64,
    pub rate: f64,
    pub cluster: String,
    pub timezone: String,
    pub port: i64,
    pub status: LimitProfileStatus,
}

impl LimitProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitProfileStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitProfileStatus) -> bool {
        match self.status {
            LimitProfileStatus::Proposed => {
                match next {
                    LimitProfileStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitProfileStatus::Approved => {
                match next {
                    LimitProfileStatus::Rejected | LimitProfileStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitProfileStatus::Rejected => {
                return next == LimitProfileStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.minimum - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitProfile {
        LimitProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            amount: self.amount,
            notes: self.notes.clone(),
            minimum: self.minimum,
            rate: self.rate,
            cluster: self.cluster.clone(),
            timezone: self.timezone.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a limit profile in status proposed.
pub fn new_limit_profile(code: &str, label: &str) -> LimitProfile {
    LimitProfile {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        amount: 0.0,
        notes: "".to_string(),
        minimum: 0.0,
        rate: 0.0,
        cluster: "".to_string(),
        timezone: "".to_string(),
        port: 0,
        status: LimitProfileStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_limit_profile("code-922", "label-629");
        assert_eq!(item.status, LimitProfileStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_limit_profile("code-922", "label-629");
        if item.move_to(LimitProfileStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitProfileStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_limit_profile("code-922", "label-629");
        item.status = LimitProfileStatus::Withdrawn;
        assert!(
            item.move_to(LimitProfileStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
