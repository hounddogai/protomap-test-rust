use super::*;

use acme_kit::text;

/// Where a audit event score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AuditEventScoreStatus {
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

impl std::fmt::Display for AuditEventScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event score as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventScore {
    pub code: String,
    pub title: String,
    pub category: String,
    pub limit: i64,
    pub source: String,
    pub amount: f64,
    pub capacity: i64,
    pub currency: String,
    pub reviewer: String,
    pub score: f64,
    pub status: AuditEventScoreStatus,
}

impl AuditEventScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventScoreStatus) -> bool {
        match self.status {
            AuditEventScoreStatus::New => {
                match next {
                    AuditEventScoreStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventScoreStatus::Queued => {
                match next {
                    AuditEventScoreStatus::Running | AuditEventScoreStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventScoreStatus::Running => {
                match next {
                    AuditEventScoreStatus::Done | AuditEventScoreStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventScoreStatus::Done => {
                return next == AuditEventScoreStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.capacity as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventScore {
        AuditEventScore {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            limit: self.limit,
            source: self.source.clone(),
            amount: self.amount,
            capacity: self.capacity,
            currency: self.currency.clone(),
            reviewer: self.reviewer.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a audit event score in status new.
pub fn new_audit_event_score(code: &str, title: &str) -> AuditEventScore {
    AuditEventScore {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        limit: 0,
        source: "".to_string(),
        amount: 0.0,
        capacity: 0,
        currency: "".to_string(),
        reviewer: "".to_string(),
        score: 0.0,
        status: AuditEventScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_audit_event_score("code-946", "title-76");
        assert_eq!(item.status, AuditEventScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_audit_event_score("code-946", "title-76");
        if item.move_to(AuditEventScoreStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventScoreStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_audit_event_score("code-946", "title-76");
        item.status = AuditEventScoreStatus::Failed;
        assert!(
            item.move_to(AuditEventScoreStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
