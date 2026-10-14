use super::*;

use acme_kit::text;

/// Where a primary nonce notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryNonceNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryNonceNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryNonceNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary nonce notice as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryNonceNotice {
    pub code: String,
    pub label: String,
    pub region: String,
    pub description: String,
    pub priority: i64,
    pub version: i64,
    pub currency: String,
    pub archived: bool,
    pub title: String,
    pub threshold: f64,
    pub status: PrimaryNonceNoticeStatus,
}

impl PrimaryNonceNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryNonceNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryNonceNoticeStatus) -> bool {
        match self.status {
            PrimaryNonceNoticeStatus::Proposed => {
                match next {
                    PrimaryNonceNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryNonceNoticeStatus::Approved => {
                match next {
                    PrimaryNonceNoticeStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryNonceNoticeStatus::Rejected => {
                match next {
                    PrimaryNonceNoticeStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PrimaryNonceNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary nonce notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        (self.threshold * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryNonceNotice {
        PrimaryNonceNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            description: self.description.clone(),
            priority: self.priority,
            version: self.version,
            currency: self.currency.clone(),
            archived: self.archived,
            title: self.title.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary nonce notice in status proposed.
pub fn new_primary_nonce_notice(code: &str, label: &str) -> PrimaryNonceNotice {
    PrimaryNonceNotice {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        description: "".to_string(),
        priority: 0,
        version: 0,
        currency: "".to_string(),
        archived: false,
        title: "".to_string(),
        threshold: 0.0,
        status: PrimaryNonceNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_nonce_notice("code-66", "label-475");
        assert_eq!(item.status, PrimaryNonceNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_primary_nonce_notice("code-66", "label-475");
        if item.move_to(PrimaryNonceNoticeStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryNonceNoticeStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_nonce_notice("code-66", "label-475");
        item.status = PrimaryNonceNoticeStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryNonceNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
