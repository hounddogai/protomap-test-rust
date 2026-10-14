use super::*;

use acme_kit::text;

/// Where a rollout quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RolloutQuotaStatus {
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

impl std::fmt::Display for RolloutQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutQuota {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub reviewer: String,
    pub minimum: f64,
    pub floor_value: f64,
    pub maximum: f64,
    pub host: String,
    pub scope: String,
    pub status: RolloutQuotaStatus,
}

impl RolloutQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutQuotaStatus) -> bool {
        match self.status {
            RolloutQuotaStatus::Draft => {
                match next {
                    RolloutQuotaStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutQuotaStatus::Pending => {
                match next {
                    RolloutQuotaStatus::Active | RolloutQuotaStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutQuotaStatus::Active => {
                return next == RolloutQuotaStatus::Suspended || next == RolloutQuotaStatus::Closed;
            }
            RolloutQuotaStatus::Suspended => {
                match next {
                    RolloutQuotaStatus::Closed => {
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

    pub fn move_to(&mut self, next: RolloutQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollout quota", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.minimum + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutQuota {
        RolloutQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            reviewer: self.reviewer.clone(),
            minimum: self.minimum,
            floor_value: self.floor_value,
            maximum: self.maximum,
            host: self.host.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a rollout quota in status draft.
pub fn new_rollout_quota(code: &str, name: &str) -> RolloutQuota {
    RolloutQuota {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        reviewer: "".to_string(),
        minimum: 0.0,
        floor_value: 0.0,
        maximum: 0.0,
        host: "".to_string(),
        scope: "".to_string(),
        status: RolloutQuotaStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rollout_quota("code-94", "name-15");
        assert_eq!(item.status, RolloutQuotaStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_rollout_quota("code-94", "name-15");
        if item.move_to(RolloutQuotaStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutQuotaStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rollout_quota("code-94", "name-15");
        item.status = RolloutQuotaStatus::Closed;
        assert!(
            item.move_to(RolloutQuotaStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
