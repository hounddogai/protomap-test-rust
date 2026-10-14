use super::*;

use acme_kit::text;

/// Where a alert manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AlertManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AlertManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertManifest {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub version: i64,
    pub created_at: i64,
    pub phone: String,
    pub window_days: i64,
    pub budget: f64,
    pub label: String,
    pub minimum: f64,
    pub revision: i64,
    pub status: AlertManifestStatus,
}

impl AlertManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertManifestStatus) -> bool {
        match self.status {
            AlertManifestStatus::Created => {
                return next == AlertManifestStatus::Retired;
            }
            AlertManifestStatus::Validated => {
                match next {
                    AlertManifestStatus::Published | AlertManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertManifestStatus::Published => {
                return next == AlertManifestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert manifest {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "alert manifest", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.version as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertManifest {
        AlertManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            version: self.version,
            created_at: self.created_at,
            phone: self.phone.clone(),
            window_days: self.window_days,
            budget: self.budget,
            label: self.label.clone(),
            minimum: self.minimum,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a alert manifest in status created.
pub fn new_alert_manifest(code: &str, name: &str) -> AlertManifest {
    AlertManifest {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        version: 0,
        created_at: 0,
        phone: "".to_string(),
        window_days: 0,
        budget: 0.0,
        label: "".to_string(),
        minimum: 0.0,
        revision: 0,
        status: AlertManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_alert_manifest("code-781", "name-236");
        assert_eq!(item.status, AlertManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_alert_manifest("code-781", "name-236");
        if item.move_to(AlertManifestStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertManifestStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_alert_manifest("code-781", "name-236");
        item.status = AlertManifestStatus::Retired;
        assert!(
            item.move_to(AlertManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
