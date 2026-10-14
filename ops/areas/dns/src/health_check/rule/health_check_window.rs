use super::*;

use acme_kit::text;

/// Where a health check window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckWindowStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl HealthCheckWindowStatus {
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

impl std::fmt::Display for HealthCheckWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a health check window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckWindow {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub starts_at: i64,
    pub active: bool,
    pub namespace: String,
    pub ceiling: f64,
    pub region: String,
    pub category: String,
    pub status: HealthCheckWindowStatus,
}

impl HealthCheckWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckWindowStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckWindowStatus) -> bool {
        match self.status {
            HealthCheckWindowStatus::Open => {
                return next == HealthCheckWindowStatus::InReview;
            }
            HealthCheckWindowStatus::InReview => {
                return next == HealthCheckWindowStatus::Reopened || next == HealthCheckWindowStatus::Archived;
            }
            HealthCheckWindowStatus::Resolved => {
                return next == HealthCheckWindowStatus::Archived;
            }
            HealthCheckWindowStatus::Reopened => {
                match next {
                    HealthCheckWindowStatus::Archived => {
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

    pub fn move_to(&mut self, next: HealthCheckWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ceiling - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckWindow {
        HealthCheckWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            starts_at: self.starts_at,
            active: self.active,
            namespace: self.namespace.clone(),
            ceiling: self.ceiling,
            region: self.region.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a health check window in status open.
pub fn new_health_check_window(code: &str, label: &str) -> HealthCheckWindow {
    HealthCheckWindow {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        starts_at: 0,
        active: false,
        namespace: "".to_string(),
        ceiling: 0.0,
        region: "".to_string(),
        category: "".to_string(),
        status: HealthCheckWindowStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_health_check_window("code-975", "label-613");
        assert_eq!(item.status, HealthCheckWindowStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_health_check_window("code-975", "label-613");
        if item.move_to(HealthCheckWindowStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckWindowStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_health_check_window("code-975", "label-613");
        item.status = HealthCheckWindowStatus::Archived;
        assert!(
            item.move_to(HealthCheckWindowStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
