use super::*;

use acme_kit::text;

/// Where a tenant adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantAdjustmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TenantAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for TenantAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantAdjustment {
    pub code: String,
    pub title: String,
    pub category: String,
    pub quota_units: i64,
    pub floor_value: f64,
    pub budget: f64,
    pub country: String,
    pub retries: i64,
    pub revision: i64,
    pub status: TenantAdjustmentStatus,
}

impl TenantAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantAdjustmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantAdjustmentStatus) -> bool {
        match self.status {
            TenantAdjustmentStatus::Idle => {
                match next {
                    TenantAdjustmentStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantAdjustmentStatus::Warming => {
                match next {
                    TenantAdjustmentStatus::Ready | TenantAdjustmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantAdjustmentStatus::Ready => {
                return next == TenantAdjustmentStatus::Stopped;
            }
            TenantAdjustmentStatus::Draining => {
                match next {
                    TenantAdjustmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TenantAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.quota_units as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantAdjustment {
        TenantAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            quota_units: self.quota_units,
            floor_value: self.floor_value,
            budget: self.budget,
            country: self.country.clone(),
            retries: self.retries,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tenant adjustment in status idle.
pub fn new_tenant_adjustment(code: &str, title: &str) -> TenantAdjustment {
    TenantAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        quota_units: 0,
        floor_value: 0.0,
        budget: 0.0,
        country: "".to_string(),
        retries: 0,
        revision: 0,
        status: TenantAdjustmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tenant_adjustment("code-366", "title-422");
        assert_eq!(item.status, TenantAdjustmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_tenant_adjustment("code-366", "title-422");
        if item.move_to(TenantAdjustmentStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantAdjustmentStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tenant_adjustment("code-366", "title-422");
        item.status = TenantAdjustmentStatus::Stopped;
        assert!(
            item.move_to(TenantAdjustmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
