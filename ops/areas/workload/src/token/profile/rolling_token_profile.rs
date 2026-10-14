use super::*;

use acme_kit::text;

/// Where a rolling token profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTokenProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingTokenProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RollingTokenProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling token profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenProfile {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub grace_days: i64,
    pub issuer: String,
    pub checksum: String,
    pub approved: bool,
    pub status: RollingTokenProfileStatus,
}

impl RollingTokenProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTokenProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTokenProfileStatus) -> bool {
        match self.status {
            RollingTokenProfileStatus::New => {
                match next {
                    RollingTokenProfileStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTokenProfileStatus::Queued => {
                return next == RollingTokenProfileStatus::Done;
            }
            RollingTokenProfileStatus::Running => {
                return next == RollingTokenProfileStatus::Done || next == RollingTokenProfileStatus::Failed;
            }
            RollingTokenProfileStatus::Done => {
                match next {
                    RollingTokenProfileStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingTokenProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling token profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.score - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingTokenProfile {
        RollingTokenProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            grace_days: self.grace_days,
            issuer: self.issuer.clone(),
            checksum: self.checksum.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling token profile in status new.
pub fn new_rolling_token_profile(code: &str, label: &str) -> RollingTokenProfile {
    RollingTokenProfile {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        grace_days: 0,
        issuer: "".to_string(),
        checksum: "".to_string(),
        approved: false,
        status: RollingTokenProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_token_profile("code-482", "label-560");
        assert_eq!(item.status, RollingTokenProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_rolling_token_profile("code-482", "label-560");
        if item.move_to(RollingTokenProfileStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingTokenProfileStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_token_profile("code-482", "label-560");
        item.status = RollingTokenProfileStatus::Failed;
        assert!(
            item.move_to(RollingTokenProfileStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
