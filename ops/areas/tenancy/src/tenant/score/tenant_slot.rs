use super::*;

use acme_kit::text;

/// Where a tenant slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TenantSlotStatus {
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

impl std::fmt::Display for TenantSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tenant slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantSlot {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub principal: String,
    pub tenant_id: String,
    pub burst: i64,
    pub status: TenantSlotStatus,
}

impl TenantSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantSlotStatus) -> bool {
        match self.status {
            TenantSlotStatus::Planned => {
                match next {
                    TenantSlotStatus::InProgress | TenantSlotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantSlotStatus::Scheduled => {
                match next {
                    TenantSlotStatus::InProgress | TenantSlotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantSlotStatus::InProgress => {
                return next == TenantSlotStatus::Completed || next == TenantSlotStatus::Cancelled;
            }
            TenantSlotStatus::Completed => {
                return next == TenantSlotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tenant slot", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.expires_at as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantSlot {
        TenantSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            principal: self.principal.clone(),
            tenant_id: self.tenant_id.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tenant slot in status planned.
pub fn new_tenant_slot(code: &str, name: &str) -> TenantSlot {
    TenantSlot {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        principal: "".to_string(),
        tenant_id: "".to_string(),
        burst: 0,
        status: TenantSlotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tenant_slot("code-435", "name-76");
        assert_eq!(item.status, TenantSlotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_tenant_slot("code-435", "name-76");
        if item.move_to(TenantSlotStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantSlotStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tenant_slot("code-435", "name-76");
        item.status = TenantSlotStatus::Cancelled;
        assert!(
            item.move_to(TenantSlotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
