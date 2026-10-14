use super::*;

use acme_kit::text;

/// Where a rolling approval variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingApprovalVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingApprovalVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RollingApprovalVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling approval variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalVariance {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub unit: String,
    pub ratio: f64,
    pub tenant_id: String,
    pub created_at: i64,
    pub country: String,
    pub archived: bool,
    pub status: RollingApprovalVarianceStatus,
}

impl RollingApprovalVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingApprovalVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingApprovalVarianceStatus) -> bool {
        match self.status {
            RollingApprovalVarianceStatus::New => {
                match next {
                    RollingApprovalVarianceStatus::Running | RollingApprovalVarianceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingApprovalVarianceStatus::Queued => {
                match next {
                    RollingApprovalVarianceStatus::Running | RollingApprovalVarianceStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingApprovalVarianceStatus::Running => {
                return next == RollingApprovalVarianceStatus::Failed;
            }
            RollingApprovalVarianceStatus::Done => {
                return next == RollingApprovalVarianceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingApprovalVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling approval variance {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.cost + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingApprovalVariance {
        RollingApprovalVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            unit: self.unit.clone(),
            ratio: self.ratio,
            tenant_id: self.tenant_id.clone(),
            created_at: self.created_at,
            country: self.country.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rolling approval variance in status new.
pub fn new_rolling_approval_variance(code: &str, label: &str) -> RollingApprovalVariance {
    RollingApprovalVariance {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        unit: "".to_string(),
        ratio: 0.0,
        tenant_id: "".to_string(),
        created_at: 0,
        country: "".to_string(),
        archived: false,
        status: RollingApprovalVarianceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_approval_variance("code-559", "label-16");
        assert_eq!(item.status, RollingApprovalVarianceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_rolling_approval_variance("code-559", "label-16");
        if item.move_to(RollingApprovalVarianceStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingApprovalVarianceStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_approval_variance("code-559", "label-16");
        item.status = RollingApprovalVarianceStatus::Failed;
        assert!(
            item.move_to(RollingApprovalVarianceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
