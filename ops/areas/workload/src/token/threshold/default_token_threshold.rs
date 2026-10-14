use super::*;

use acme_kit::text;

/// Where a default token threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultTokenThresholdStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DefaultTokenThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DefaultTokenThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default token threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTokenThreshold {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub region: String,
    pub updated_at: i64,
    pub pinned: bool,
    pub source: String,
    pub active: bool,
    pub status: DefaultTokenThresholdStatus,
}

impl DefaultTokenThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultTokenThresholdStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultTokenThresholdStatus) -> bool {
        match self.status {
            DefaultTokenThresholdStatus::Created => {
                match next {
                    DefaultTokenThresholdStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTokenThresholdStatus::Validated => {
                return next == DefaultTokenThresholdStatus::Published;
            }
            DefaultTokenThresholdStatus::Published => {
                match next {
                    DefaultTokenThresholdStatus::Retired => {
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

    pub fn move_to(&mut self, next: DefaultTokenThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default token threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultTokenThreshold {
        DefaultTokenThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            region: self.region.clone(),
            updated_at: self.updated_at,
            pinned: self.pinned,
            source: self.source.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a default token threshold in status created.
pub fn new_default_token_threshold(code: &str, name: &str) -> DefaultTokenThreshold {
    DefaultTokenThreshold {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        region: "".to_string(),
        updated_at: 0,
        pinned: false,
        source: "".to_string(),
        active: false,
        status: DefaultTokenThresholdStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_default_token_threshold("code-458", "name-215");
        assert_eq!(item.status, DefaultTokenThresholdStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_default_token_threshold("code-458", "name-215");
        if item.move_to(DefaultTokenThresholdStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultTokenThresholdStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_default_token_threshold("code-458", "name-215");
        item.status = DefaultTokenThresholdStatus::Retired;
        assert!(
            item.move_to(DefaultTokenThresholdStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
