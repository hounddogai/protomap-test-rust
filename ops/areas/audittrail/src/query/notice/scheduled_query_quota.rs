use super::*;

use acme_kit::text;

/// Where a scheduled query quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledQueryQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledQueryQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledQueryQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled query quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryQuota {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub margin: f64,
    pub active: bool,
    pub score: f64,
    pub cluster: String,
    pub key_id: String,
    pub region: String,
    pub status: ScheduledQueryQuotaStatus,
}

impl ScheduledQueryQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledQueryQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledQueryQuotaStatus) -> bool {
        match self.status {
            ScheduledQueryQuotaStatus::Proposed => {
                return next == ScheduledQueryQuotaStatus::Approved;
            }
            ScheduledQueryQuotaStatus::Approved => {
                return next == ScheduledQueryQuotaStatus::Rejected || next == ScheduledQueryQuotaStatus::Withdrawn;
            }
            ScheduledQueryQuotaStatus::Rejected => {
                match next {
                    ScheduledQueryQuotaStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ScheduledQueryQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled query quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        (self.margin * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledQueryQuota {
        ScheduledQueryQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            margin: self.margin,
            active: self.active,
            score: self.score,
            cluster: self.cluster.clone(),
            key_id: self.key_id.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a scheduled query quota in status proposed.
pub fn new_scheduled_query_quota(code: &str, title: &str) -> ScheduledQueryQuota {
    ScheduledQueryQuota {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        margin: 0.0,
        active: false,
        score: 0.0,
        cluster: "".to_string(),
        key_id: "".to_string(),
        region: "".to_string(),
        status: ScheduledQueryQuotaStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_query_quota("code-472", "title-196");
        assert_eq!(item.status, ScheduledQueryQuotaStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_scheduled_query_quota("code-472", "title-196");
        if item.move_to(ScheduledQueryQuotaStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledQueryQuotaStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_query_quota("code-472", "title-196");
        item.status = ScheduledQueryQuotaStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledQueryQuotaStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
