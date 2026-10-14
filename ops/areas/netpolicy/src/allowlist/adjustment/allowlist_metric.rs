use super::*;

use acme_kit::text;

/// Where a allowlist metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistMetricStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AllowlistMetricStatus {
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

impl std::fmt::Display for AllowlistMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMetric {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub expires_at: i64,
    pub rate: f64,
    pub tenant_id: String,
    pub status: AllowlistMetricStatus,
}

impl AllowlistMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistMetricStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistMetricStatus) -> bool {
        match self.status {
            AllowlistMetricStatus::Planned => {
                match next {
                    AllowlistMetricStatus::Completed | AllowlistMetricStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistMetricStatus::Scheduled => {
                match next {
                    AllowlistMetricStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistMetricStatus::InProgress => {
                return next == AllowlistMetricStatus::Completed;
            }
            AllowlistMetricStatus::Completed => {
                match next {
                    AllowlistMetricStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: AllowlistMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist metric {} cannot move from {} to {}",
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
        format!("{} {}", "allowlist metric", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.floor_value - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistMetric {
        AllowlistMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            expires_at: self.expires_at,
            rate: self.rate,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a allowlist metric in status planned.
pub fn new_allowlist_metric(code: &str, name: &str) -> AllowlistMetric {
    AllowlistMetric {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        expires_at: 0,
        rate: 0.0,
        tenant_id: "".to_string(),
        status: AllowlistMetricStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_allowlist_metric("code-296", "name-282");
        assert_eq!(item.status, AllowlistMetricStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_allowlist_metric("code-296", "name-282");
        if item.move_to(AllowlistMetricStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistMetricStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_allowlist_metric("code-296", "name-282");
        item.status = AllowlistMetricStatus::Cancelled;
        assert!(
            item.move_to(AllowlistMetricStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
