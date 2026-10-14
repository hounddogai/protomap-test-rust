use super::*;

use acme_kit::text;

/// Where a escalation quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EscalationQuotaStatus {
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

impl std::fmt::Display for EscalationQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation quota as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationQuota {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub updated_at: i64,
    pub ratio: f64,
    pub archived: bool,
    pub limit: i64,
    pub priority: i64,
    pub label: String,
    pub status: EscalationQuotaStatus,
}

impl EscalationQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationQuotaStatus) -> bool {
        match self.status {
            EscalationQuotaStatus::New => {
                match next {
                    EscalationQuotaStatus::Done | EscalationQuotaStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationQuotaStatus::Queued => {
                match next {
                    EscalationQuotaStatus::Running | EscalationQuotaStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationQuotaStatus::Running => {
                return next == EscalationQuotaStatus::Failed;
            }
            EscalationQuotaStatus::Done => {
                return next == EscalationQuotaStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "escalation quota", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.updated_at as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationQuota {
        EscalationQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            updated_at: self.updated_at,
            ratio: self.ratio,
            archived: self.archived,
            limit: self.limit,
            priority: self.priority,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a escalation quota in status new.
pub fn new_escalation_quota(code: &str, title: &str) -> EscalationQuota {
    EscalationQuota {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        updated_at: 0,
        ratio: 0.0,
        archived: false,
        limit: 0,
        priority: 0,
        label: "".to_string(),
        status: EscalationQuotaStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_escalation_quota("code-326", "title-390");
        assert_eq!(item.status, EscalationQuotaStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_escalation_quota("code-326", "title-390");
        if item.move_to(EscalationQuotaStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationQuotaStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_escalation_quota("code-326", "title-390");
        item.status = EscalationQuotaStatus::Failed;
        assert!(
            item.move_to(EscalationQuotaStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
