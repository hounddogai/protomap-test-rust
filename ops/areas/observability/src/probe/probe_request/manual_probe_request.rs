use super::*;

use acme_kit::text;

/// Where a manual probe request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualProbeRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualProbeRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ManualProbeRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual probe request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeRequest {
    pub code: String,
    pub name: String,
    pub title: String,
    pub fingerprint: String,
    pub source: String,
    pub ceiling: f64,
    pub status: ManualProbeRequestStatus,
}

impl ManualProbeRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualProbeRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualProbeRequestStatus) -> bool {
        match self.status {
            ManualProbeRequestStatus::New => {
                return next == ManualProbeRequestStatus::Done;
            }
            ManualProbeRequestStatus::Queued => {
                return next == ManualProbeRequestStatus::Done || next == ManualProbeRequestStatus::Failed;
            }
            ManualProbeRequestStatus::Running => {
                match next {
                    ManualProbeRequestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualProbeRequestStatus::Done => {
                match next {
                    ManualProbeRequestStatus::Failed => {
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

    pub fn move_to(&mut self, next: ManualProbeRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual probe request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> ManualProbeRequest {
        ManualProbeRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            source: self.source.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a manual probe request in status new.
pub fn new_manual_probe_request(code: &str, name: &str) -> ManualProbeRequest {
    ManualProbeRequest {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        fingerprint: "".to_string(),
        source: "".to_string(),
        ceiling: 0.0,
        status: ManualProbeRequestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_probe_request("code-551", "name-440");
        assert_eq!(item.status, ManualProbeRequestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_manual_probe_request("code-551", "name-440");
        if item.move_to(ManualProbeRequestStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualProbeRequestStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_probe_request("code-551", "name-440");
        item.status = ManualProbeRequestStatus::Failed;
        assert!(
            item.move_to(ManualProbeRequestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
