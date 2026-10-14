use super::*;

use acme_kit::text;

/// Where a isolation slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IsolationSlotStatus {
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

impl std::fmt::Display for IsolationSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation slot as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSlot {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub visible: bool,
    pub title: String,
    pub ends_at: i64,
    pub owner_id: String,
    pub checksum: String,
    pub contact_email: String,
    pub status: IsolationSlotStatus,
}

impl IsolationSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationSlotStatus) -> bool {
        match self.status {
            IsolationSlotStatus::Planned => {
                match next {
                    IsolationSlotStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationSlotStatus::Scheduled => {
                match next {
                    IsolationSlotStatus::InProgress | IsolationSlotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationSlotStatus::InProgress => {
                return next == IsolationSlotStatus::Completed;
            }
            IsolationSlotStatus::Completed => {
                match next {
                    IsolationSlotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: IsolationSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ends_at as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationSlot {
        IsolationSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            visible: self.visible,
            title: self.title.clone(),
            ends_at: self.ends_at,
            owner_id: self.owner_id.clone(),
            checksum: self.checksum.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a isolation slot in status planned.
pub fn new_isolation_slot(code: &str, name: &str) -> IsolationSlot {
    IsolationSlot {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        visible: false,
        title: "".to_string(),
        ends_at: 0,
        owner_id: "".to_string(),
        checksum: "".to_string(),
        contact_email: "".to_string(),
        status: IsolationSlotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_isolation_slot("code-594", "name-268");
        assert_eq!(item.status, IsolationSlotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_isolation_slot("code-594", "name-268");
        if item.move_to(IsolationSlotStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationSlotStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_isolation_slot("code-594", "name-268");
        item.status = IsolationSlotStatus::Cancelled;
        assert!(
            item.move_to(IsolationSlotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
