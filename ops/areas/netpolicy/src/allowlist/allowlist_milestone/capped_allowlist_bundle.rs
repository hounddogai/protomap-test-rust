use super::*;

use acme_kit::text;

/// Where a capped allowlist bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedAllowlistBundleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedAllowlistBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedAllowlistBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped allowlist bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedAllowlistBundle {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub discount: f64,
    pub country: String,
    pub approved: bool,
    pub cluster: String,
    pub minimum: f64,
    pub scope: String,
    pub checksum: String,
    pub status: CappedAllowlistBundleStatus,
}

impl CappedAllowlistBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedAllowlistBundleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedAllowlistBundleStatus) -> bool {
        match self.status {
            CappedAllowlistBundleStatus::Created => {
                return next == CappedAllowlistBundleStatus::Validated || next == CappedAllowlistBundleStatus::Retired;
            }
            CappedAllowlistBundleStatus::Validated => {
                return next == CappedAllowlistBundleStatus::Published;
            }
            CappedAllowlistBundleStatus::Published => {
                return next == CappedAllowlistBundleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedAllowlistBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped allowlist bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.minimum - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedAllowlistBundle {
        CappedAllowlistBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            discount: self.discount,
            country: self.country.clone(),
            approved: self.approved,
            cluster: self.cluster.clone(),
            minimum: self.minimum,
            scope: self.scope.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a capped allowlist bundle in status created.
pub fn new_capped_allowlist_bundle(code: &str, title: &str) -> CappedAllowlistBundle {
    CappedAllowlistBundle {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        discount: 0.0,
        country: "".to_string(),
        approved: false,
        cluster: "".to_string(),
        minimum: 0.0,
        scope: "".to_string(),
        checksum: "".to_string(),
        status: CappedAllowlistBundleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_allowlist_bundle("code-299", "title-770");
        assert_eq!(item.status, CappedAllowlistBundleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_capped_allowlist_bundle("code-299", "title-770");
        if item.move_to(CappedAllowlistBundleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedAllowlistBundleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_allowlist_bundle("code-299", "title-770");
        item.status = CappedAllowlistBundleStatus::Retired;
        assert!(
            item.move_to(CappedAllowlistBundleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
