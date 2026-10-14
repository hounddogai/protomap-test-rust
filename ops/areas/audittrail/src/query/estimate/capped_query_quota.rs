use super::*;

use acme_kit::text;

/// Where a capped query quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedQueryQuotaStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedQueryQuotaStatus {
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

impl std::fmt::Display for CappedQueryQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped query quota as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryQuota {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub ceiling: f64,
    pub floor_value: f64,
    pub region: String,
    pub burst: i64,
    pub timezone: String,
    pub status: CappedQueryQuotaStatus,
}

impl CappedQueryQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedQueryQuotaStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedQueryQuotaStatus) -> bool {
        match self.status {
            CappedQueryQuotaStatus::Planned => {
                match next {
                    CappedQueryQuotaStatus::Scheduled | CappedQueryQuotaStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedQueryQuotaStatus::Scheduled => {
                match next {
                    CappedQueryQuotaStatus::Completed | CappedQueryQuotaStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedQueryQuotaStatus::InProgress => {
                match next {
                    CappedQueryQuotaStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedQueryQuotaStatus::Completed => {
                match next {
                    CappedQueryQuotaStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: CappedQueryQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped query quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ceiling + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedQueryQuota {
        CappedQueryQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            ceiling: self.ceiling,
            floor_value: self.floor_value,
            region: self.region.clone(),
            burst: self.burst,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a capped query quota in status planned.
pub fn new_capped_query_quota(code: &str, label: &str) -> CappedQueryQuota {
    CappedQueryQuota {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        ceiling: 0.0,
        floor_value: 0.0,
        region: "".to_string(),
        burst: 0,
        timezone: "".to_string(),
        status: CappedQueryQuotaStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_capped_query_quota("code-828", "label-112");
        assert_eq!(item.status, CappedQueryQuotaStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_capped_query_quota("code-828", "label-112");
        if item.move_to(CappedQueryQuotaStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedQueryQuotaStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_capped_query_quota("code-828", "label-112");
        item.status = CappedQueryQuotaStatus::Cancelled;
        assert!(
            item.move_to(CappedQueryQuotaStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
