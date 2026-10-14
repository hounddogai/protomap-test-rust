use super::*;

use acme_kit::text;

/// Where a tiered zone plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredZonePlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredZonePlanStatus {
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

impl std::fmt::Display for TieredZonePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered zone plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredZonePlan {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub duration_minutes: i64,
    pub created_at: i64,
    pub ttl_seconds: i64,
    pub status: TieredZonePlanStatus,
}

impl TieredZonePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredZonePlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredZonePlanStatus) -> bool {
        match self.status {
            TieredZonePlanStatus::Draft => {
                return next == TieredZonePlanStatus::Suspended;
            }
            TieredZonePlanStatus::Pending => {
                match next {
                    TieredZonePlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredZonePlanStatus::Active => {
                return next == TieredZonePlanStatus::Suspended;
            }
            TieredZonePlanStatus::Suspended => {
                match next {
                    TieredZonePlanStatus::Closed => {
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

    pub fn move_to(&mut self, next: TieredZonePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered zone plan {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.created_at as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredZonePlan {
        TieredZonePlan {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            duration_minutes: self.duration_minutes,
            created_at: self.created_at,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered zone plan in status draft.
pub fn new_tiered_zone_plan(code: &str, title: &str) -> TieredZonePlan {
    TieredZonePlan {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        duration_minutes: 0,
        created_at: 0,
        ttl_seconds: 0,
        status: TieredZonePlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_zone_plan("code-418", "title-152");
        assert_eq!(item.status, TieredZonePlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_tiered_zone_plan("code-418", "title-152");
        if item.move_to(TieredZonePlanStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredZonePlanStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_zone_plan("code-418", "title-152");
        item.status = TieredZonePlanStatus::Closed;
        assert!(
            item.move_to(TieredZonePlanStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
