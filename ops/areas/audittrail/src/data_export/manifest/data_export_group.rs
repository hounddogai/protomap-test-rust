use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a data export group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DataExportGroupStatus {
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

impl std::fmt::Display for DataExportGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportGroup {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub attributes: BTreeMap<String, String>,
    pub description: String,
    pub rate: f64,
    pub limit: i64,
    pub grace_days: i64,
    pub retries: i64,
    pub status: DataExportGroupStatus,
}

impl DataExportGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportGroupStatus) -> bool {
        match self.status {
            DataExportGroupStatus::Idle => {
                match next {
                    DataExportGroupStatus::Ready | DataExportGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportGroupStatus::Warming => {
                return next == DataExportGroupStatus::Draining || next == DataExportGroupStatus::Stopped;
            }
            DataExportGroupStatus::Ready => {
                return next == DataExportGroupStatus::Stopped;
            }
            DataExportGroupStatus::Draining => {
                return next == DataExportGroupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn effective_total(&self) -> f64 {
        ((self.limit as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportGroup {
        DataExportGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            attributes: self.attributes.clone(),
            description: self.description.clone(),
            rate: self.rate,
            limit: self.limit,
            grace_days: self.grace_days,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a data export group in status idle.
pub fn new_data_export_group(code: &str, title: &str) -> DataExportGroup {
    DataExportGroup {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        attributes: BTreeMap::new(),
        description: "".to_string(),
        rate: 0.0,
        limit: 0,
        grace_days: 0,
        retries: 0,
        status: DataExportGroupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_data_export_group("code-797", "title-510");
        assert_eq!(item.status, DataExportGroupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_data_export_group("code-797", "title-510");
        if item.move_to(DataExportGroupStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportGroupStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_data_export_group("code-797", "title-510");
        item.status = DataExportGroupStatus::Stopped;
        assert!(
            item.move_to(DataExportGroupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
