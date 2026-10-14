use super::*;

use acme_kit::text;

/// Where a segment manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SegmentManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SegmentManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One segment manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentManifest {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub updated_at: i64,
    pub pinned: bool,
    pub checksum: String,
    pub unit: String,
    pub external_ref: String,
    pub created_at: i64,
    pub issuer: String,
    pub status: SegmentManifestStatus,
}

impl SegmentManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentManifestStatus) -> bool {
        match self.status {
            SegmentManifestStatus::Created => {
                return next == SegmentManifestStatus::Validated;
            }
            SegmentManifestStatus::Validated => {
                return next == SegmentManifestStatus::Retired;
            }
            SegmentManifestStatus::Published => {
                match next {
                    SegmentManifestStatus::Retired => {
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

    pub fn move_to(&mut self, next: SegmentManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment manifest {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "segment manifest", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.updated_at as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentManifest {
        SegmentManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            updated_at: self.updated_at,
            pinned: self.pinned,
            checksum: self.checksum.clone(),
            unit: self.unit.clone(),
            external_ref: self.external_ref.clone(),
            created_at: self.created_at,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a segment manifest in status created.
pub fn new_segment_manifest(code: &str, label: &str) -> SegmentManifest {
    SegmentManifest {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        updated_at: 0,
        pinned: false,
        checksum: "".to_string(),
        unit: "".to_string(),
        external_ref: "".to_string(),
        created_at: 0,
        issuer: "".to_string(),
        status: SegmentManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_segment_manifest("code-748", "label-779");
        assert_eq!(item.status, SegmentManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_segment_manifest("code-748", "label-779");
        if item.move_to(SegmentManifestStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentManifestStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_segment_manifest("code-748", "label-779");
        item.status = SegmentManifestStatus::Retired;
        assert!(
            item.move_to(SegmentManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
