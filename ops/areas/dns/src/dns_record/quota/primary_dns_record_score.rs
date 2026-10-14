use super::*;

use acme_kit::text;

/// Where a primary dns record score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryDnsRecordScoreStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryDnsRecordScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PrimaryDnsRecordScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary dns record score as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordScore {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub port: i64,
    pub visible: bool,
    pub external_ref: String,
    pub rotation_days: i64,
    pub phone: String,
    pub status: PrimaryDnsRecordScoreStatus,
}

impl PrimaryDnsRecordScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryDnsRecordScoreStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryDnsRecordScoreStatus) -> bool {
        match self.status {
            PrimaryDnsRecordScoreStatus::Planned => {
                match next {
                    PrimaryDnsRecordScoreStatus::InProgress | PrimaryDnsRecordScoreStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDnsRecordScoreStatus::Scheduled => {
                match next {
                    PrimaryDnsRecordScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDnsRecordScoreStatus::InProgress => {
                return next == PrimaryDnsRecordScoreStatus::Completed
                    || next == PrimaryDnsRecordScoreStatus::Cancelled;
            }
            PrimaryDnsRecordScoreStatus::Completed => {
                return next == PrimaryDnsRecordScoreStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryDnsRecordScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary dns record score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.port as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryDnsRecordScore {
        PrimaryDnsRecordScore {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            port: self.port,
            visible: self.visible,
            external_ref: self.external_ref.clone(),
            rotation_days: self.rotation_days,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary dns record score in status planned.
pub fn new_primary_dns_record_score(code: &str, title: &str) -> PrimaryDnsRecordScore {
    PrimaryDnsRecordScore {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        port: 0,
        visible: false,
        external_ref: "".to_string(),
        rotation_days: 0,
        phone: "".to_string(),
        status: PrimaryDnsRecordScoreStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_dns_record_score("code-479", "title-81");
        assert_eq!(item.status, PrimaryDnsRecordScoreStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_primary_dns_record_score("code-479", "title-81");
        if item.move_to(PrimaryDnsRecordScoreStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryDnsRecordScoreStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_dns_record_score("code-479", "title-81");
        item.status = PrimaryDnsRecordScoreStatus::Cancelled;
        assert!(
            item.move_to(PrimaryDnsRecordScoreStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
