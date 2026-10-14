use super::*;

use acme_kit::text;

/// Where a fallback region pin mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRegionPinMappingStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackRegionPinMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackRegionPinMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback region pin mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinMapping {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub created_at: i64,
    pub source: String,
    pub title: String,
    pub ends_at: i64,
    pub checksum: String,
    pub status: FallbackRegionPinMappingStatus,
}

impl FallbackRegionPinMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRegionPinMappingStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRegionPinMappingStatus) -> bool {
        match self.status {
            FallbackRegionPinMappingStatus::Created => {
                return next == FallbackRegionPinMappingStatus::Validated;
            }
            FallbackRegionPinMappingStatus::Validated => {
                match next {
                    FallbackRegionPinMappingStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRegionPinMappingStatus::Published => {
                match next {
                    FallbackRegionPinMappingStatus::Retired => {
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

    pub fn move_to(&mut self, next: FallbackRegionPinMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback region pin mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback region pin mapping", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ends_at as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRegionPinMapping {
        FallbackRegionPinMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            created_at: self.created_at,
            source: self.source.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a fallback region pin mapping in status created.
pub fn new_fallback_region_pin_mapping(code: &str, label: &str) -> FallbackRegionPinMapping {
    FallbackRegionPinMapping {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        created_at: 0,
        source: "".to_string(),
        title: "".to_string(),
        ends_at: 0,
        checksum: "".to_string(),
        status: FallbackRegionPinMappingStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_region_pin_mapping("code-687", "label-671");
        assert_eq!(item.status, FallbackRegionPinMappingStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_fallback_region_pin_mapping("code-687", "label-671");
        if item.move_to(FallbackRegionPinMappingStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRegionPinMappingStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_region_pin_mapping("code-687", "label-671");
        item.status = FallbackRegionPinMappingStatus::Retired;
        assert!(
            item.move_to(FallbackRegionPinMappingStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
