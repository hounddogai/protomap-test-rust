use super::*;

use acme_kit::text;

/// Where a manual postmortem request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualPostmortemRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualPostmortemRequestStatus {
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

impl std::fmt::Display for ManualPostmortemRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual postmortem request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPostmortemRequest {
    pub code: String,
    pub title: String,
    pub country: String,
    pub phone: String,
    pub rate: f64,
    pub tags: Vec<String>,
    pub version: i64,
    pub status: ManualPostmortemRequestStatus,
}

impl ManualPostmortemRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualPostmortemRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualPostmortemRequestStatus) -> bool {
        match self.status {
            ManualPostmortemRequestStatus::New => {
                return next == ManualPostmortemRequestStatus::Running || next == ManualPostmortemRequestStatus::Failed;
            }
            ManualPostmortemRequestStatus::Queued => {
                return next == ManualPostmortemRequestStatus::Running;
            }
            ManualPostmortemRequestStatus::Running => {
                match next {
                    ManualPostmortemRequestStatus::Done | ManualPostmortemRequestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualPostmortemRequestStatus::Done => {
                match next {
                    ManualPostmortemRequestStatus::Failed => {
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

    pub fn move_to(&mut self, next: ManualPostmortemRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual postmortem request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.version as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualPostmortemRequest {
        ManualPostmortemRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            phone: self.phone.clone(),
            rate: self.rate,
            tags: self.tags.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a manual postmortem request in status new.
pub fn new_manual_postmortem_request(code: &str, title: &str) -> ManualPostmortemRequest {
    ManualPostmortemRequest {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        phone: "".to_string(),
        rate: 0.0,
        tags: Vec::new(),
        version: 0,
        status: ManualPostmortemRequestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_postmortem_request("code-791", "title-764");
        assert_eq!(item.status, ManualPostmortemRequestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_manual_postmortem_request("code-791", "title-764");
        if item.move_to(ManualPostmortemRequestStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualPostmortemRequestStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_postmortem_request("code-791", "title-764");
        item.status = ManualPostmortemRequestStatus::Failed;
        assert!(
            item.move_to(ManualPostmortemRequestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
