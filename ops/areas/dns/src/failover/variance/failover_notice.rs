use super::*;

use acme_kit::text;

/// Where a failover notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FailoverNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FailoverNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover notice as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverNotice {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub title: String,
    pub audience: String,
    pub visible: bool,
    pub key_id: String,
    pub pinned: bool,
    pub status: FailoverNoticeStatus,
}

impl FailoverNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverNoticeStatus) -> bool {
        match self.status {
            FailoverNoticeStatus::Proposed => {
                match next {
                    FailoverNoticeStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverNoticeStatus::Approved => {
                match next {
                    FailoverNoticeStatus::Rejected | FailoverNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverNoticeStatus::Rejected => {
                match next {
                    FailoverNoticeStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FailoverNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> FailoverNotice {
        FailoverNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            title: self.title.clone(),
            audience: self.audience.clone(),
            visible: self.visible,
            key_id: self.key_id.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a failover notice in status proposed.
pub fn new_failover_notice(code: &str, label: &str) -> FailoverNotice {
    FailoverNotice {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        title: "".to_string(),
        audience: "".to_string(),
        visible: false,
        key_id: "".to_string(),
        pinned: false,
        status: FailoverNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_failover_notice("code-935", "label-376");
        assert_eq!(item.status, FailoverNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_failover_notice("code-935", "label-376");
        if item.move_to(FailoverNoticeStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverNoticeStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_failover_notice("code-935", "label-376");
        item.status = FailoverNoticeStatus::Withdrawn;
        assert!(
            item.move_to(FailoverNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
