use super::*;

use acme_kit::text;

/// Where a tiered binding bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredBindingBundleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredBindingBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredBindingBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered binding bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingBundle {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub quota_units: i64,
    pub approved: bool,
    pub tags: Vec<String>,
    pub scope: String,
    pub starts_at: i64,
    pub status: TieredBindingBundleStatus,
}

impl TieredBindingBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredBindingBundleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredBindingBundleStatus) -> bool {
        match self.status {
            TieredBindingBundleStatus::Created => {
                return next == TieredBindingBundleStatus::Validated;
            }
            TieredBindingBundleStatus::Validated => {
                return next == TieredBindingBundleStatus::Published || next == TieredBindingBundleStatus::Retired;
            }
            TieredBindingBundleStatus::Published => {
                return next == TieredBindingBundleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredBindingBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered binding bundle {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "tiered binding bundle", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.quota_units as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredBindingBundle {
        TieredBindingBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            quota_units: self.quota_units,
            approved: self.approved,
            tags: self.tags.clone(),
            scope: self.scope.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tiered binding bundle in status created.
pub fn new_tiered_binding_bundle(code: &str, title: &str) -> TieredBindingBundle {
    TieredBindingBundle {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        quota_units: 0,
        approved: false,
        tags: Vec::new(),
        scope: "".to_string(),
        starts_at: 0,
        status: TieredBindingBundleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_binding_bundle("code-666", "title-201");
        assert_eq!(item.status, TieredBindingBundleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tiered_binding_bundle("code-666", "title-201");
        if item.move_to(TieredBindingBundleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredBindingBundleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_binding_bundle("code-666", "title-201");
        item.status = TieredBindingBundleStatus::Retired;
        assert!(
            item.move_to(TieredBindingBundleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
