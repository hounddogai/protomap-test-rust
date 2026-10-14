use super::*;

use acme_kit::text;

/// Where a manual data export notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDataExportNoticeStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualDataExportNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualDataExportNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual data export notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportNotice {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub title: String,
    pub visible: bool,
    pub starts_at: i64,
    pub phone: String,
    pub sequence: i64,
    pub key_id: String,
    pub contact_email: String,
    pub approved: bool,
    pub status: ManualDataExportNoticeStatus,
}

impl ManualDataExportNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDataExportNoticeStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDataExportNoticeStatus) -> bool {
        match self.status {
            ManualDataExportNoticeStatus::Requested => {
                match next {
                    ManualDataExportNoticeStatus::Granted | ManualDataExportNoticeStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDataExportNoticeStatus::Granted => {
                match next {
                    ManualDataExportNoticeStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDataExportNoticeStatus::Revoked => {
                match next {
                    ManualDataExportNoticeStatus::Expired => {
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

    pub fn move_to(&mut self, next: ManualDataExportNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual data export notice {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.amount * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualDataExportNotice {
        ManualDataExportNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            title: self.title.clone(),
            visible: self.visible,
            starts_at: self.starts_at,
            phone: self.phone.clone(),
            sequence: self.sequence,
            key_id: self.key_id.clone(),
            contact_email: self.contact_email.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual data export notice in status requested.
pub fn new_manual_data_export_notice(code: &str, label: &str) -> ManualDataExportNotice {
    ManualDataExportNotice {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        title: "".to_string(),
        visible: false,
        starts_at: 0,
        phone: "".to_string(),
        sequence: 0,
        key_id: "".to_string(),
        contact_email: "".to_string(),
        approved: false,
        status: ManualDataExportNoticeStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_data_export_notice("code-230", "label-370");
        assert_eq!(item.status, ManualDataExportNoticeStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_manual_data_export_notice("code-230", "label-370");
        if item.move_to(ManualDataExportNoticeStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualDataExportNoticeStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_data_export_notice("code-230", "label-370");
        item.status = ManualDataExportNoticeStatus::Expired;
        assert!(
            item.move_to(ManualDataExportNoticeStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
