use super::*;

use acme_kit::text;

/// Where a tenant plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantPlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TenantPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for TenantPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantPlan {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub created_at: i64,
    pub summary: String,
    pub cluster: String,
    pub grace_days: i64,
    pub tags: Vec<String>,
    pub ends_at: i64,
    pub status: TenantPlanStatus,
}

impl TenantPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantPlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantPlanStatus) -> bool {
        match self.status {
            TenantPlanStatus::Open => {
                return next == TenantPlanStatus::InReview;
            }
            TenantPlanStatus::InReview => {
                match next {
                    TenantPlanStatus::Reopened | TenantPlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantPlanStatus::Resolved => {
                match next {
                    TenantPlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantPlanStatus::Reopened => {
                match next {
                    TenantPlanStatus::Archived => {
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

    pub fn move_to(&mut self, next: TenantPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant plan {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.grace_days as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantPlan {
        TenantPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            created_at: self.created_at,
            summary: self.summary.clone(),
            cluster: self.cluster.clone(),
            grace_days: self.grace_days,
            tags: self.tags.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a tenant plan in status open.
pub fn new_tenant_plan(code: &str, label: &str) -> TenantPlan {
    TenantPlan {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        created_at: 0,
        summary: "".to_string(),
        cluster: "".to_string(),
        grace_days: 0,
        tags: Vec::new(),
        ends_at: 0,
        status: TenantPlanStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tenant_plan("code-254", "label-845");
        assert_eq!(item.status, TenantPlanStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_tenant_plan("code-254", "label-845");
        if item.move_to(TenantPlanStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantPlanStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tenant_plan("code-254", "label-845");
        item.status = TenantPlanStatus::Archived;
        assert!(item.move_to(TenantPlanStatus::Open).is_err(), "expected move to fail");
    }
}
