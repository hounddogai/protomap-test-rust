use super::*;

use acme_kit::text;

/// Where a seasonal slo manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalSloManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalSloManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalSloManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal slo manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloManifest {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub locale: String,
    pub tags: Vec<String>,
    pub threshold: f64,
    pub duration_minutes: i64,
    pub cluster: String,
    pub weight: f64,
    pub visible: bool,
    pub expires_at: i64,
    pub status: SeasonalSloManifestStatus,
}

impl SeasonalSloManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalSloManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalSloManifestStatus) -> bool {
        match self.status {
            SeasonalSloManifestStatus::Created => {
                return next == SeasonalSloManifestStatus::Validated;
            }
            SeasonalSloManifestStatus::Validated => {
                match next {
                    SeasonalSloManifestStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalSloManifestStatus::Published => {
                return next == SeasonalSloManifestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalSloManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal slo manifest {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.starts_at as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalSloManifest {
        SeasonalSloManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            locale: self.locale.clone(),
            tags: self.tags.clone(),
            threshold: self.threshold,
            duration_minutes: self.duration_minutes,
            cluster: self.cluster.clone(),
            weight: self.weight,
            visible: self.visible,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a seasonal slo manifest in status created.
pub fn new_seasonal_slo_manifest(code: &str, label: &str) -> SeasonalSloManifest {
    SeasonalSloManifest {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        locale: "".to_string(),
        tags: Vec::new(),
        threshold: 0.0,
        duration_minutes: 0,
        cluster: "".to_string(),
        weight: 0.0,
        visible: false,
        expires_at: 0,
        status: SeasonalSloManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_seasonal_slo_manifest("code-906", "label-612");
        assert_eq!(item.status, SeasonalSloManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_seasonal_slo_manifest("code-906", "label-612");
        if item.move_to(SeasonalSloManifestStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalSloManifestStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_seasonal_slo_manifest("code-906", "label-612");
        item.status = SeasonalSloManifestStatus::Retired;
        assert!(
            item.move_to(SeasonalSloManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
