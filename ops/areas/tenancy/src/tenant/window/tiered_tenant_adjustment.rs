use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a tiered tenant adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredTenantAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredTenantAdjustmentStatus {
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

impl std::fmt::Display for TieredTenantAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered tenant adjustment as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantAdjustment {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub issuer: String,
    pub margin: f64,
    pub sequence: i64,
    pub region: String,
    pub threshold: f64,
    pub created_at: i64,
    pub floor_value: f64,
    pub status: TieredTenantAdjustmentStatus,
}

impl TieredTenantAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredTenantAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredTenantAdjustmentStatus) -> bool {
        match self.status {
            TieredTenantAdjustmentStatus::Planned => {
                match next {
                    TieredTenantAdjustmentStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredTenantAdjustmentStatus::Scheduled => {
                return next == TieredTenantAdjustmentStatus::InProgress
                    || next == TieredTenantAdjustmentStatus::Completed;
            }
            TieredTenantAdjustmentStatus::InProgress => {
                match next {
                    TieredTenantAdjustmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredTenantAdjustmentStatus::Completed => {
                return next == TieredTenantAdjustmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredTenantAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered tenant adjustment {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_load(&self) -> f64 {
        ((self.created_at as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredTenantAdjustment {
        TieredTenantAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            attributes: self.attributes.clone(),
            issuer: self.issuer.clone(),
            margin: self.margin,
            sequence: self.sequence,
            region: self.region.clone(),
            threshold: self.threshold,
            created_at: self.created_at,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tiered tenant adjustment in status planned.
pub fn new_tiered_tenant_adjustment(code: &str, title: &str) -> TieredTenantAdjustment {
    TieredTenantAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        attributes: BTreeMap::new(),
        issuer: "".to_string(),
        margin: 0.0,
        sequence: 0,
        region: "".to_string(),
        threshold: 0.0,
        created_at: 0,
        floor_value: 0.0,
        status: TieredTenantAdjustmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_tenant_adjustment("code-75", "title-146");
        assert_eq!(item.status, TieredTenantAdjustmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_tiered_tenant_adjustment("code-75", "title-146");
        if item.move_to(TieredTenantAdjustmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredTenantAdjustmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_tenant_adjustment("code-75", "title-146");
        item.status = TieredTenantAdjustmentStatus::Cancelled;
        assert!(
            item.move_to(TieredTenantAdjustmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
