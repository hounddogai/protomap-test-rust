use super::*;

use acme_kit::text;

/// Where a isolation group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IsolationGroupStatus {
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

impl std::fmt::Display for IsolationGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a isolation group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationGroup {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub updated_at: i64,
    pub minimum: f64,
    pub revision: i64,
    pub fingerprint: String,
    pub status: IsolationGroupStatus,
}

impl IsolationGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationGroupStatus) -> bool {
        match self.status {
            IsolationGroupStatus::Planned => {
                return next == IsolationGroupStatus::InProgress;
            }
            IsolationGroupStatus::Scheduled => {
                match next {
                    IsolationGroupStatus::InProgress | IsolationGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationGroupStatus::InProgress => {
                return next == IsolationGroupStatus::Cancelled;
            }
            IsolationGroupStatus::Completed => {
                match next {
                    IsolationGroupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: IsolationGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.cost - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationGroup {
        IsolationGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            updated_at: self.updated_at,
            minimum: self.minimum,
            revision: self.revision,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a isolation group in status planned.
pub fn new_isolation_group(code: &str, label: &str) -> IsolationGroup {
    IsolationGroup {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        updated_at: 0,
        minimum: 0.0,
        revision: 0,
        fingerprint: "".to_string(),
        status: IsolationGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_isolation_group("code-195", "label-227");
        assert_eq!(item.status, IsolationGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_isolation_group("code-195", "label-227");
        if item.move_to(IsolationGroupStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationGroupStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_isolation_group("code-195", "label-227");
        item.status = IsolationGroupStatus::Cancelled;
        assert!(
            item.move_to(IsolationGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
