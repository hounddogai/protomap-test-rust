use super::*;

use acme_kit::text;

/// Where a default dashboard digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDashboardDigestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DefaultDashboardDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DefaultDashboardDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default dashboard digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardDigest {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub notes: String,
    pub created_at: i64,
    pub currency: String,
    pub floor_value: f64,
    pub port: i64,
    pub budget: f64,
    pub grace_days: i64,
    pub status: DefaultDashboardDigestStatus,
}

impl DefaultDashboardDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDashboardDigestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDashboardDigestStatus) -> bool {
        match self.status {
            DefaultDashboardDigestStatus::Created => {
                match next {
                    DefaultDashboardDigestStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultDashboardDigestStatus::Validated => {
                return next == DefaultDashboardDigestStatus::Published
                    || next == DefaultDashboardDigestStatus::Retired;
            }
            DefaultDashboardDigestStatus::Published => {
                match next {
                    DefaultDashboardDigestStatus::Retired => {
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

    pub fn move_to(&mut self, next: DefaultDashboardDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default dashboard digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.port as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultDashboardDigest {
        DefaultDashboardDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            notes: self.notes.clone(),
            created_at: self.created_at,
            currency: self.currency.clone(),
            floor_value: self.floor_value,
            port: self.port,
            budget: self.budget,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default dashboard digest in status created.
pub fn new_default_dashboard_digest(code: &str, title: &str) -> DefaultDashboardDigest {
    DefaultDashboardDigest {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        notes: "".to_string(),
        created_at: 0,
        currency: "".to_string(),
        floor_value: 0.0,
        port: 0,
        budget: 0.0,
        grace_days: 0,
        status: DefaultDashboardDigestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_default_dashboard_digest("code-274", "title-858");
        assert_eq!(item.status, DefaultDashboardDigestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_default_dashboard_digest("code-274", "title-858");
        if item.move_to(DefaultDashboardDigestStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultDashboardDigestStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_default_dashboard_digest("code-274", "title-858");
        item.status = DefaultDashboardDigestStatus::Retired;
        assert!(
            item.move_to(DefaultDashboardDigestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
