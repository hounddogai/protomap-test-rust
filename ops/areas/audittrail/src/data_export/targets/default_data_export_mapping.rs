use super::*;

use acme_kit::text;

/// Where a default data export mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDataExportMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultDataExportMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultDataExportMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default data export mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportMapping {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub name: String,
    pub namespace: String,
    pub category: String,
    pub ceiling: f64,
    pub contact_email: String,
    pub scope: String,
    pub updated_at: i64,
    pub rate: f64,
    pub status: DefaultDataExportMappingStatus,
}

impl DefaultDataExportMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDataExportMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDataExportMappingStatus) -> bool {
        match self.status {
            DefaultDataExportMappingStatus::Requested => {
                match next {
                    DefaultDataExportMappingStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultDataExportMappingStatus::Granted => {
                return next == DefaultDataExportMappingStatus::Revoked;
            }
            DefaultDataExportMappingStatus::Revoked => {
                return next == DefaultDataExportMappingStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultDataExportMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default data export mapping {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        (self.rate * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultDataExportMapping {
        DefaultDataExportMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            category: self.category.clone(),
            ceiling: self.ceiling,
            contact_email: self.contact_email.clone(),
            scope: self.scope.clone(),
            updated_at: self.updated_at,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a default data export mapping in status requested.
pub fn new_default_data_export_mapping(code: &str, label: &str) -> DefaultDataExportMapping {
    DefaultDataExportMapping {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        name: "".to_string(),
        namespace: "".to_string(),
        category: "".to_string(),
        ceiling: 0.0,
        contact_email: "".to_string(),
        scope: "".to_string(),
        updated_at: 0,
        rate: 0.0,
        status: DefaultDataExportMappingStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_data_export_mapping("code-780", "label-49");
        assert_eq!(item.status, DefaultDataExportMappingStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_default_data_export_mapping("code-780", "label-49");
        if item.move_to(DefaultDataExportMappingStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultDataExportMappingStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_data_export_mapping("code-780", "label-49");
        item.status = DefaultDataExportMappingStatus::Expired;
        assert!(
            item.move_to(DefaultDataExportMappingStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
