use super::*;

use acme_kit::text;

/// Where a pending binding quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingBindingQuotaStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingBindingQuotaStatus {
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

impl std::fmt::Display for PendingBindingQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending binding quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingQuota {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub cluster: String,
    pub locale: String,
    pub archived: bool,
    pub cipher: String,
    pub capacity: i64,
    pub status: PendingBindingQuotaStatus,
}

impl PendingBindingQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingBindingQuotaStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingBindingQuotaStatus) -> bool {
        match self.status {
            PendingBindingQuotaStatus::Planned => {
                match next {
                    PendingBindingQuotaStatus::InProgress | PendingBindingQuotaStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingBindingQuotaStatus::Scheduled => {
                match next {
                    PendingBindingQuotaStatus::InProgress | PendingBindingQuotaStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingBindingQuotaStatus::InProgress => {
                return next == PendingBindingQuotaStatus::Completed || next == PendingBindingQuotaStatus::Cancelled;
            }
            PendingBindingQuotaStatus::Completed => {
                match next {
                    PendingBindingQuotaStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PendingBindingQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending binding quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.capacity as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingBindingQuota {
        PendingBindingQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            cluster: self.cluster.clone(),
            locale: self.locale.clone(),
            archived: self.archived,
            cipher: self.cipher.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a pending binding quota in status planned.
pub fn new_pending_binding_quota(code: &str, name: &str) -> PendingBindingQuota {
    PendingBindingQuota {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        cluster: "".to_string(),
        locale: "".to_string(),
        archived: false,
        cipher: "".to_string(),
        capacity: 0,
        status: PendingBindingQuotaStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_pending_binding_quota("code-697", "name-649");
        assert_eq!(item.status, PendingBindingQuotaStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_pending_binding_quota("code-697", "name-649");
        if item.move_to(PendingBindingQuotaStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingBindingQuotaStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_pending_binding_quota("code-697", "name-649");
        item.status = PendingBindingQuotaStatus::Cancelled;
        assert!(
            item.move_to(PendingBindingQuotaStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
