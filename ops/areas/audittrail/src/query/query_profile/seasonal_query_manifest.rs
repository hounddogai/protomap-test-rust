use super::*;

use acme_kit::text;

/// Where a seasonal query manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalQueryManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalQueryManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalQueryManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal query manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryManifest {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub ratio: f64,
    pub category: String,
    pub discount: f64,
    pub version: i64,
    pub maximum: f64,
    pub rotation_days: i64,
    pub expires_at: i64,
    pub status: SeasonalQueryManifestStatus,
}

impl SeasonalQueryManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalQueryManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalQueryManifestStatus) -> bool {
        match self.status {
            SeasonalQueryManifestStatus::Created => {
                match next {
                    SeasonalQueryManifestStatus::Published | SeasonalQueryManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalQueryManifestStatus::Validated => {
                return next == SeasonalQueryManifestStatus::Retired;
            }
            SeasonalQueryManifestStatus::Published => {
                return next == SeasonalQueryManifestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalQueryManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal query manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.maximum + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalQueryManifest {
        SeasonalQueryManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            ratio: self.ratio,
            category: self.category.clone(),
            discount: self.discount,
            version: self.version,
            maximum: self.maximum,
            rotation_days: self.rotation_days,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a seasonal query manifest in status created.
pub fn new_seasonal_query_manifest(code: &str, title: &str) -> SeasonalQueryManifest {
    SeasonalQueryManifest {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        ratio: 0.0,
        category: "".to_string(),
        discount: 0.0,
        version: 0,
        maximum: 0.0,
        rotation_days: 0,
        expires_at: 0,
        status: SeasonalQueryManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_seasonal_query_manifest("code-262", "title-974");
        assert_eq!(item.status, SeasonalQueryManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_seasonal_query_manifest("code-262", "title-974");
        if item.move_to(SeasonalQueryManifestStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalQueryManifestStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_seasonal_query_manifest("code-262", "title-974");
        item.status = SeasonalQueryManifestStatus::Retired;
        assert!(
            item.move_to(SeasonalQueryManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
