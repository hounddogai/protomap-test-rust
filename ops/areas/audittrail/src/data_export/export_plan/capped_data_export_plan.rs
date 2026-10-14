use super::*;

use acme_kit::text;

/// Where a capped data export plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedDataExportPlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedDataExportPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedDataExportPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped data export plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportPlan {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub sequence: i64,
    pub burst: i64,
    pub quantity: i64,
    pub expires_at: i64,
    pub tenant_id: String,
    pub status: CappedDataExportPlanStatus,
}

impl CappedDataExportPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedDataExportPlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedDataExportPlanStatus) -> bool {
        match self.status {
            CappedDataExportPlanStatus::Requested => {
                match next {
                    CappedDataExportPlanStatus::Granted | CappedDataExportPlanStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedDataExportPlanStatus::Granted => {
                match next {
                    CappedDataExportPlanStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedDataExportPlanStatus::Revoked => {
                return next == CappedDataExportPlanStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedDataExportPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped data export plan {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.expires_at as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedDataExportPlan {
        CappedDataExportPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            sequence: self.sequence,
            burst: self.burst,
            quantity: self.quantity,
            expires_at: self.expires_at,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped data export plan in status requested.
pub fn new_capped_data_export_plan(code: &str, title: &str) -> CappedDataExportPlan {
    CappedDataExportPlan {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        sequence: 0,
        burst: 0,
        quantity: 0,
        expires_at: 0,
        tenant_id: "".to_string(),
        status: CappedDataExportPlanStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_data_export_plan("code-291", "title-789");
        assert_eq!(item.status, CappedDataExportPlanStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_capped_data_export_plan("code-291", "title-789");
        if item.move_to(CappedDataExportPlanStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedDataExportPlanStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_data_export_plan("code-291", "title-789");
        item.status = CappedDataExportPlanStatus::Expired;
        assert!(
            item.move_to(CappedDataExportPlanStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
