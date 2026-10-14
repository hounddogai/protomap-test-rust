use super::*;

use acme_kit::text;

/// Where a pending health check group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingHealthCheckGroupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingHealthCheckGroupStatus {
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

impl std::fmt::Display for PendingHealthCheckGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending health check group as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckGroup {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub expires_at: i64,
    pub rate: f64,
    pub status: PendingHealthCheckGroupStatus,
}

impl PendingHealthCheckGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingHealthCheckGroupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingHealthCheckGroupStatus) -> bool {
        match self.status {
            PendingHealthCheckGroupStatus::Draft => {
                match next {
                    PendingHealthCheckGroupStatus::Pending | PendingHealthCheckGroupStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingHealthCheckGroupStatus::Pending => {
                return next == PendingHealthCheckGroupStatus::Closed;
            }
            PendingHealthCheckGroupStatus::Active => {
                match next {
                    PendingHealthCheckGroupStatus::Suspended | PendingHealthCheckGroupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingHealthCheckGroupStatus::Suspended => {
                return next == PendingHealthCheckGroupStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingHealthCheckGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending health check group {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        (self.weight * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingHealthCheckGroup {
        PendingHealthCheckGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            expires_at: self.expires_at,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending health check group in status draft.
pub fn new_pending_health_check_group(code: &str, name: &str) -> PendingHealthCheckGroup {
    PendingHealthCheckGroup {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        expires_at: 0,
        rate: 0.0,
        status: PendingHealthCheckGroupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_health_check_group("code-964", "name-678");
        assert_eq!(item.status, PendingHealthCheckGroupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_pending_health_check_group("code-964", "name-678");
        if item.move_to(PendingHealthCheckGroupStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingHealthCheckGroupStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_health_check_group("code-964", "name-678");
        item.status = PendingHealthCheckGroupStatus::Closed;
        assert!(
            item.move_to(PendingHealthCheckGroupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
