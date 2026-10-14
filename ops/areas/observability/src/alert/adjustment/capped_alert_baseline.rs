use super::*;

use acme_kit::text;

/// Where a capped alert baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedAlertBaselineStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedAlertBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedAlertBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped alert baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedAlertBaseline {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub starts_at: i64,
    pub rate: f64,
    pub port: i64,
    pub limit: i64,
    pub active: bool,
    pub name: String,
    pub reviewer: String,
    pub version: i64,
    pub status: CappedAlertBaselineStatus,
}

impl CappedAlertBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedAlertBaselineStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedAlertBaselineStatus) -> bool {
        match self.status {
            CappedAlertBaselineStatus::Created => {
                match next {
                    CappedAlertBaselineStatus::Validated | CappedAlertBaselineStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedAlertBaselineStatus::Validated => {
                return next == CappedAlertBaselineStatus::Published;
            }
            CappedAlertBaselineStatus::Published => {
                match next {
                    CappedAlertBaselineStatus::Retired => {
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

    pub fn move_to(&mut self, next: CappedAlertBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped alert baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.starts_at as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedAlertBaseline {
        CappedAlertBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            starts_at: self.starts_at,
            rate: self.rate,
            port: self.port,
            limit: self.limit,
            active: self.active,
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped alert baseline in status created.
pub fn new_capped_alert_baseline(code: &str, title: &str) -> CappedAlertBaseline {
    CappedAlertBaseline {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        starts_at: 0,
        rate: 0.0,
        port: 0,
        limit: 0,
        active: false,
        name: "".to_string(),
        reviewer: "".to_string(),
        version: 0,
        status: CappedAlertBaselineStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_alert_baseline("code-411", "title-853");
        assert_eq!(item.status, CappedAlertBaselineStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_capped_alert_baseline("code-411", "title-853");
        if item.move_to(CappedAlertBaselineStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedAlertBaselineStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_alert_baseline("code-411", "title-853");
        item.status = CappedAlertBaselineStatus::Retired;
        assert!(
            item.move_to(CappedAlertBaselineStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
