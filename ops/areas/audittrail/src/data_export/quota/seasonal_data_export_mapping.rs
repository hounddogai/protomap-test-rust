use super::*;

use acme_kit::text;

/// Where a seasonal data export mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalDataExportMappingStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalDataExportMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalDataExportMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal data export mapping as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportMapping {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub cost: f64,
    pub label: String,
    pub expires_at: i64,
    pub status: SeasonalDataExportMappingStatus,
}

impl SeasonalDataExportMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalDataExportMappingStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalDataExportMappingStatus) -> bool {
        match self.status {
            SeasonalDataExportMappingStatus::Created => {
                return next == SeasonalDataExportMappingStatus::Validated;
            }
            SeasonalDataExportMappingStatus::Validated => {
                match next {
                    SeasonalDataExportMappingStatus::Published | SeasonalDataExportMappingStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDataExportMappingStatus::Published => {
                return next == SeasonalDataExportMappingStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalDataExportMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal data export mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal data export mapping", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.minimum - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalDataExportMapping {
        SeasonalDataExportMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            cost: self.cost,
            label: self.label.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal data export mapping in status created.
pub fn new_seasonal_data_export_mapping(code: &str, name: &str) -> SeasonalDataExportMapping {
    SeasonalDataExportMapping {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        cost: 0.0,
        label: "".to_string(),
        expires_at: 0,
        status: SeasonalDataExportMappingStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_seasonal_data_export_mapping("code-186", "name-76");
        assert_eq!(item.status, SeasonalDataExportMappingStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_seasonal_data_export_mapping("code-186", "name-76");
        if item.move_to(SeasonalDataExportMappingStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalDataExportMappingStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_seasonal_data_export_mapping("code-186", "name-76");
        item.status = SeasonalDataExportMappingStatus::Retired;
        assert!(
            item.move_to(SeasonalDataExportMappingStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
