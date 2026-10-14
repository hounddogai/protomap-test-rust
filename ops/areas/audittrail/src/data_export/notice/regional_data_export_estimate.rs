use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a regional data export estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalDataExportEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalDataExportEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RegionalDataExportEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional data export estimate as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportEstimate {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub attributes: BTreeMap<String, String>,
    pub category: String,
    pub starts_at: i64,
    pub quota_units: i64,
    pub approved: bool,
    pub status: RegionalDataExportEstimateStatus,
}

impl RegionalDataExportEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalDataExportEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalDataExportEstimateStatus) -> bool {
        match self.status {
            RegionalDataExportEstimateStatus::Idle => {
                match next {
                    RegionalDataExportEstimateStatus::Warming | RegionalDataExportEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDataExportEstimateStatus::Warming => {
                match next {
                    RegionalDataExportEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDataExportEstimateStatus::Ready => {
                match next {
                    RegionalDataExportEstimateStatus::Draining | RegionalDataExportEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDataExportEstimateStatus::Draining => {
                return next == RegionalDataExportEstimateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalDataExportEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional data export estimate {} cannot move from {} to {}",
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
        format!("{} {}", "regional data export estimate", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.quota_units as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalDataExportEstimate {
        RegionalDataExportEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            attributes: self.attributes.clone(),
            category: self.category.clone(),
            starts_at: self.starts_at,
            quota_units: self.quota_units,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a regional data export estimate in status idle.
pub fn new_regional_data_export_estimate(code: &str, label: &str) -> RegionalDataExportEstimate {
    RegionalDataExportEstimate {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        attributes: BTreeMap::new(),
        category: "".to_string(),
        starts_at: 0,
        quota_units: 0,
        approved: false,
        status: RegionalDataExportEstimateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_data_export_estimate("code-604", "label-361");
        assert_eq!(item.status, RegionalDataExportEstimateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_regional_data_export_estimate("code-604", "label-361");
        if item.move_to(RegionalDataExportEstimateStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalDataExportEstimateStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_data_export_estimate("code-604", "label-361");
        item.status = RegionalDataExportEstimateStatus::Stopped;
        assert!(
            item.move_to(RegionalDataExportEstimateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
