use super::*;

use acme_kit::text;

/// Where a retention tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionTierStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RetentionTierStatus {
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

impl std::fmt::Display for RetentionTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionTier {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub host: String,
    pub duration_minutes: i64,
    pub summary: String,
    pub status: RetentionTierStatus,
}

impl RetentionTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionTierStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionTierStatus) -> bool {
        match self.status {
            RetentionTierStatus::Planned => {
                match next {
                    RetentionTierStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionTierStatus::Scheduled => {
                match next {
                    RetentionTierStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionTierStatus::InProgress => {
                return next == RetentionTierStatus::Completed;
            }
            RetentionTierStatus::Completed => {
                match next {
                    RetentionTierStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RetentionTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionTier {
        RetentionTier {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            host: self.host.clone(),
            duration_minutes: self.duration_minutes,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a retention tier in status planned.
pub fn new_retention_tier(code: &str, name: &str) -> RetentionTier {
    RetentionTier {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        host: "".to_string(),
        duration_minutes: 0,
        summary: "".to_string(),
        status: RetentionTierStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_retention_tier("code-268", "name-16");
        assert_eq!(item.status, RetentionTierStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_retention_tier("code-268", "name-16");
        if item.move_to(RetentionTierStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionTierStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_retention_tier("code-268", "name-16");
        item.status = RetentionTierStatus::Cancelled;
        assert!(
            item.move_to(RetentionTierStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
