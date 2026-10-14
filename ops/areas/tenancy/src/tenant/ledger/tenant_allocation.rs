use super::*;

use acme_kit::text;

/// Where a tenant allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TenantAllocationStatus {
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

impl std::fmt::Display for TenantAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantAllocation {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub issuer: String,
    pub created_at: i64,
    pub scope: String,
    pub timezone: String,
    pub batch_size: i64,
    pub pinned: bool,
    pub priority: i64,
    pub status: TenantAllocationStatus,
}

impl TenantAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantAllocationStatus) -> bool {
        match self.status {
            TenantAllocationStatus::Planned => {
                return next == TenantAllocationStatus::Completed || next == TenantAllocationStatus::Cancelled;
            }
            TenantAllocationStatus::Scheduled => {
                match next {
                    TenantAllocationStatus::InProgress | TenantAllocationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantAllocationStatus::InProgress => {
                match next {
                    TenantAllocationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantAllocationStatus::Completed => {
                return next == TenantAllocationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.priority as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantAllocation {
        TenantAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            principal: self.principal.clone(),
            issuer: self.issuer.clone(),
            created_at: self.created_at,
            scope: self.scope.clone(),
            timezone: self.timezone.clone(),
            batch_size: self.batch_size,
            pinned: self.pinned,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tenant allocation in status planned.
pub fn new_tenant_allocation(code: &str, label: &str) -> TenantAllocation {
    TenantAllocation {
        code: code.to_string(),
        label: label.to_string(),
        principal: "".to_string(),
        issuer: "".to_string(),
        created_at: 0,
        scope: "".to_string(),
        timezone: "".to_string(),
        batch_size: 0,
        pinned: false,
        priority: 0,
        status: TenantAllocationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tenant_allocation("code-247", "label-210");
        assert_eq!(item.status, TenantAllocationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_tenant_allocation("code-247", "label-210");
        if item.move_to(TenantAllocationStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantAllocationStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tenant_allocation("code-247", "label-210");
        item.status = TenantAllocationStatus::Cancelled;
        assert!(
            item.move_to(TenantAllocationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
