use super::*;

use acme_kit::text;

/// Where a pending retention score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRetentionScoreStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingRetentionScoreStatus {
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

impl std::fmt::Display for PendingRetentionScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending retention score as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionScore {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub grace_days: i64,
    pub channel: String,
    pub score: f64,
    pub cipher: String,
    pub phone: String,
    pub status: PendingRetentionScoreStatus,
}

impl PendingRetentionScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRetentionScoreStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRetentionScoreStatus) -> bool {
        match self.status {
            PendingRetentionScoreStatus::Planned => {
                return next == PendingRetentionScoreStatus::Scheduled
                    || next == PendingRetentionScoreStatus::InProgress;
            }
            PendingRetentionScoreStatus::Scheduled => {
                match next {
                    PendingRetentionScoreStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRetentionScoreStatus::InProgress => {
                match next {
                    PendingRetentionScoreStatus::Completed | PendingRetentionScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRetentionScoreStatus::Completed => {
                return next == PendingRetentionScoreStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRetentionScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending retention score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending retention score", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.score * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRetentionScore {
        PendingRetentionScore {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            grace_days: self.grace_days,
            channel: self.channel.clone(),
            score: self.score,
            cipher: self.cipher.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a pending retention score in status planned.
pub fn new_pending_retention_score(code: &str, label: &str) -> PendingRetentionScore {
    PendingRetentionScore {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        grace_days: 0,
        channel: "".to_string(),
        score: 0.0,
        cipher: "".to_string(),
        phone: "".to_string(),
        status: PendingRetentionScoreStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_pending_retention_score("code-985", "label-929");
        assert_eq!(item.status, PendingRetentionScoreStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_pending_retention_score("code-985", "label-929");
        if item.move_to(PendingRetentionScoreStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRetentionScoreStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_pending_retention_score("code-985", "label-929");
        item.status = PendingRetentionScoreStatus::Cancelled;
        assert!(
            item.move_to(PendingRetentionScoreStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
