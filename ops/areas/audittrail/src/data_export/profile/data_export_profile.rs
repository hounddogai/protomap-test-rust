use super::*;

use acme_kit::text;

/// Where a data export profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DataExportProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for DataExportProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportProfile {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub retries: i64,
    pub currency: String,
    pub cost: f64,
    pub status: DataExportProfileStatus,
}

impl DataExportProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportProfileStatus) -> bool {
        match self.status {
            DataExportProfileStatus::Draft => {
                return next == DataExportProfileStatus::Active;
            }
            DataExportProfileStatus::Pending => {
                return next == DataExportProfileStatus::Suspended || next == DataExportProfileStatus::Closed;
            }
            DataExportProfileStatus::Active => {
                return next == DataExportProfileStatus::Suspended;
            }
            DataExportProfileStatus::Suspended => {
                match next {
                    DataExportProfileStatus::Closed => {
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

    pub fn move_to(&mut self, next: DataExportProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.retries as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportProfile {
        DataExportProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            retries: self.retries,
            currency: self.currency.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a data export profile in status draft.
pub fn new_data_export_profile(code: &str, name: &str) -> DataExportProfile {
    DataExportProfile {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        retries: 0,
        currency: "".to_string(),
        cost: 0.0,
        status: DataExportProfileStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_data_export_profile("code-425", "name-783");
        assert_eq!(item.status, DataExportProfileStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_data_export_profile("code-425", "name-783");
        if item.move_to(DataExportProfileStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportProfileStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_data_export_profile("code-425", "name-783");
        item.status = DataExportProfileStatus::Closed;
        assert!(
            item.move_to(DataExportProfileStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
