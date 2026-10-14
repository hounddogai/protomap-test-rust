use super::*;

use acme_kit::text;

/// Where a pending health check window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingHealthCheckWindowStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingHealthCheckWindowStatus {
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

impl std::fmt::Display for PendingHealthCheckWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending health check window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckWindow {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub minimum: f64,
    pub weight: f64,
    pub locale: String,
    pub port: i64,
    pub status: PendingHealthCheckWindowStatus,
}

impl PendingHealthCheckWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingHealthCheckWindowStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingHealthCheckWindowStatus) -> bool {
        match self.status {
            PendingHealthCheckWindowStatus::Open => {
                match next {
                    PendingHealthCheckWindowStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingHealthCheckWindowStatus::InReview => {
                match next {
                    PendingHealthCheckWindowStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingHealthCheckWindowStatus::Resolved => {
                match next {
                    PendingHealthCheckWindowStatus::Reopened | PendingHealthCheckWindowStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingHealthCheckWindowStatus::Reopened => {
                return next == PendingHealthCheckWindowStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingHealthCheckWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending health check window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending health check window", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.minimum + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingHealthCheckWindow {
        PendingHealthCheckWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            minimum: self.minimum,
            weight: self.weight,
            locale: self.locale.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a pending health check window in status open.
pub fn new_pending_health_check_window(code: &str, name: &str) -> PendingHealthCheckWindow {
    PendingHealthCheckWindow {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        minimum: 0.0,
        weight: 0.0,
        locale: "".to_string(),
        port: 0,
        status: PendingHealthCheckWindowStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_health_check_window("code-300", "name-530");
        assert_eq!(item.status, PendingHealthCheckWindowStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_pending_health_check_window("code-300", "name-530");
        if item.move_to(PendingHealthCheckWindowStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingHealthCheckWindowStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_health_check_window("code-300", "name-530");
        item.status = PendingHealthCheckWindowStatus::Archived;
        assert!(
            item.move_to(PendingHealthCheckWindowStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
