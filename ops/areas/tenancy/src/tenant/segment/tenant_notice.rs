use super::*;

use acme_kit::text;

/// Where a tenant notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TenantNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TenantNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantNotice {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub summary: String,
    pub segment: String,
    pub score: f64,
    pub locale: String,
    pub created_at: i64,
    pub owner_id: String,
    pub cost: f64,
    pub capacity: i64,
    pub status: TenantNoticeStatus,
}

impl TenantNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantNoticeStatus) -> bool {
        match self.status {
            TenantNoticeStatus::Proposed => {
                return next == TenantNoticeStatus::Approved || next == TenantNoticeStatus::Withdrawn;
            }
            TenantNoticeStatus::Approved => {
                match next {
                    TenantNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantNoticeStatus::Rejected => {
                match next {
                    TenantNoticeStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TenantNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant notice {} cannot move from {} to {}",
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

    pub fn weighted_total(&self) -> f64 {
        (self.score * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantNotice {
        TenantNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            summary: self.summary.clone(),
            segment: self.segment.clone(),
            score: self.score,
            locale: self.locale.clone(),
            created_at: self.created_at,
            owner_id: self.owner_id.clone(),
            cost: self.cost,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tenant notice in status proposed.
pub fn new_tenant_notice(code: &str, title: &str) -> TenantNotice {
    TenantNotice {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        summary: "".to_string(),
        segment: "".to_string(),
        score: 0.0,
        locale: "".to_string(),
        created_at: 0,
        owner_id: "".to_string(),
        cost: 0.0,
        capacity: 0,
        status: TenantNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tenant_notice("code-499", "title-707");
        assert_eq!(item.status, TenantNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_tenant_notice("code-499", "title-707");
        if item.move_to(TenantNoticeStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantNoticeStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tenant_notice("code-499", "title-707");
        item.status = TenantNoticeStatus::Withdrawn;
        assert!(
            item.move_to(TenantNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
