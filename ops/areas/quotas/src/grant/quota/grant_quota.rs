use super::*;

use acme_kit::text;

/// Where a grant quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GrantQuotaStatus {
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

impl std::fmt::Display for GrantQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a grant quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantQuota {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub attempts: i64,
    pub ttl_seconds: i64,
    pub amount: f64,
    pub capacity: i64,
    pub archived: bool,
    pub floor_value: f64,
    pub ends_at: i64,
    pub country: String,
    pub status: GrantQuotaStatus,
}

impl GrantQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantQuotaStatus) -> bool {
        match self.status {
            GrantQuotaStatus::Draft => {
                match next {
                    GrantQuotaStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantQuotaStatus::Pending => {
                return next == GrantQuotaStatus::Closed;
            }
            GrantQuotaStatus::Active => {
                match next {
                    GrantQuotaStatus::Suspended | GrantQuotaStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantQuotaStatus::Suspended => {
                return next == GrantQuotaStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "grant quota", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.attempts as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantQuota {
        GrantQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            attempts: self.attempts,
            ttl_seconds: self.ttl_seconds,
            amount: self.amount,
            capacity: self.capacity,
            archived: self.archived,
            floor_value: self.floor_value,
            ends_at: self.ends_at,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a grant quota in status draft.
pub fn new_grant_quota(code: &str, label: &str) -> GrantQuota {
    GrantQuota {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        attempts: 0,
        ttl_seconds: 0,
        amount: 0.0,
        capacity: 0,
        archived: false,
        floor_value: 0.0,
        ends_at: 0,
        country: "".to_string(),
        status: GrantQuotaStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_grant_quota("code-9", "label-548");
        assert_eq!(item.status, GrantQuotaStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_grant_quota("code-9", "label-548");
        if item.move_to(GrantQuotaStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantQuotaStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_grant_quota("code-9", "label-548");
        item.status = GrantQuotaStatus::Closed;
        assert!(item.move_to(GrantQuotaStatus::Draft).is_err(), "expected move to fail");
    }
}
