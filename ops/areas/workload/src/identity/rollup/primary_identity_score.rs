use super::*;

use acme_kit::text;

/// Where a primary identity score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryIdentityScoreStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryIdentityScoreStatus {
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

impl std::fmt::Display for PrimaryIdentityScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary identity score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityScore {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub batch_size: i64,
    pub capacity: i64,
    pub status: PrimaryIdentityScoreStatus,
}

impl PrimaryIdentityScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryIdentityScoreStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryIdentityScoreStatus) -> bool {
        match self.status {
            PrimaryIdentityScoreStatus::Planned => {
                return next == PrimaryIdentityScoreStatus::Scheduled || next == PrimaryIdentityScoreStatus::Completed;
            }
            PrimaryIdentityScoreStatus::Scheduled => {
                match next {
                    PrimaryIdentityScoreStatus::InProgress | PrimaryIdentityScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryIdentityScoreStatus::InProgress => {
                match next {
                    PrimaryIdentityScoreStatus::Completed | PrimaryIdentityScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryIdentityScoreStatus::Completed => {
                return next == PrimaryIdentityScoreStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryIdentityScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary identity score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.batch_size as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryIdentityScore {
        PrimaryIdentityScore {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            batch_size: self.batch_size,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary identity score in status planned.
pub fn new_primary_identity_score(code: &str, title: &str) -> PrimaryIdentityScore {
    PrimaryIdentityScore {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        batch_size: 0,
        capacity: 0,
        status: PrimaryIdentityScoreStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_identity_score("code-703", "title-644");
        assert_eq!(item.status, PrimaryIdentityScoreStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_primary_identity_score("code-703", "title-644");
        if item.move_to(PrimaryIdentityScoreStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryIdentityScoreStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_identity_score("code-703", "title-644");
        item.status = PrimaryIdentityScoreStatus::Cancelled;
        assert!(
            item.move_to(PrimaryIdentityScoreStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
