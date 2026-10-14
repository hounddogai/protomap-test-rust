use super::*;

use acme_kit::text;

/// Where a weighted tenant claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTenantClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedTenantClaimStatus {
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

impl std::fmt::Display for WeightedTenantClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted tenant claim as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTenantClaim {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub namespace: String,
    pub quantity: i64,
    pub weight: f64,
    pub pinned: bool,
    pub expires_at: i64,
    pub ttl_seconds: i64,
    pub attempts: i64,
    pub unit: String,
    pub status: WeightedTenantClaimStatus,
}

impl WeightedTenantClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTenantClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTenantClaimStatus) -> bool {
        match self.status {
            WeightedTenantClaimStatus::Planned => {
                return next == WeightedTenantClaimStatus::Completed || next == WeightedTenantClaimStatus::Cancelled;
            }
            WeightedTenantClaimStatus::Scheduled => {
                return next == WeightedTenantClaimStatus::InProgress;
            }
            WeightedTenantClaimStatus::InProgress => {
                match next {
                    WeightedTenantClaimStatus::Completed | WeightedTenantClaimStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTenantClaimStatus::Completed => {
                return next == WeightedTenantClaimStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedTenantClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted tenant claim {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.quantity as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTenantClaim {
        WeightedTenantClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            namespace: self.namespace.clone(),
            quantity: self.quantity,
            weight: self.weight,
            pinned: self.pinned,
            expires_at: self.expires_at,
            ttl_seconds: self.ttl_seconds,
            attempts: self.attempts,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a weighted tenant claim in status planned.
pub fn new_weighted_tenant_claim(code: &str, label: &str) -> WeightedTenantClaim {
    WeightedTenantClaim {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        namespace: "".to_string(),
        quantity: 0,
        weight: 0.0,
        pinned: false,
        expires_at: 0,
        ttl_seconds: 0,
        attempts: 0,
        unit: "".to_string(),
        status: WeightedTenantClaimStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_tenant_claim("code-31", "label-701");
        assert_eq!(item.status, WeightedTenantClaimStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_weighted_tenant_claim("code-31", "label-701");
        if item.move_to(WeightedTenantClaimStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedTenantClaimStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_tenant_claim("code-31", "label-701");
        item.status = WeightedTenantClaimStatus::Cancelled;
        assert!(
            item.move_to(WeightedTenantClaimStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
