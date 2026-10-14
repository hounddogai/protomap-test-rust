use super::*;

use acme_kit::text;

/// Where a release manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ReleaseManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ReleaseManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One release manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseManifest {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub ends_at: i64,
    pub title: String,
    pub status: ReleaseManifestStatus,
}

impl ReleaseManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseManifestStatus) -> bool {
        match self.status {
            ReleaseManifestStatus::Created => {
                match next {
                    ReleaseManifestStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseManifestStatus::Validated => {
                match next {
                    ReleaseManifestStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseManifestStatus::Published => {
                return next == ReleaseManifestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "release manifest", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.weight + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseManifest {
        ReleaseManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            ends_at: self.ends_at,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a release manifest in status created.
pub fn new_release_manifest(code: &str, name: &str) -> ReleaseManifest {
    ReleaseManifest {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        ends_at: 0,
        title: "".to_string(),
        status: ReleaseManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_release_manifest("code-341", "name-679");
        assert_eq!(item.status, ReleaseManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_release_manifest("code-341", "name-679");
        if item.move_to(ReleaseManifestStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseManifestStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_release_manifest("code-341", "name-679");
        item.status = ReleaseManifestStatus::Retired;
        assert!(
            item.move_to(ReleaseManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
