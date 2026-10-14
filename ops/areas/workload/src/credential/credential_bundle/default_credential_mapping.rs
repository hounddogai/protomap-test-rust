use super::*;

use acme_kit::text;

/// Where a default credential mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultCredentialMappingStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultCredentialMappingStatus {
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

impl std::fmt::Display for DefaultCredentialMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default credential mapping as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialMapping {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub summary: String,
    pub retries: i64,
    pub country: String,
    pub status: DefaultCredentialMappingStatus,
}

impl DefaultCredentialMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultCredentialMappingStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultCredentialMappingStatus) -> bool {
        match self.status {
            DefaultCredentialMappingStatus::New => {
                match next {
                    DefaultCredentialMappingStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultCredentialMappingStatus::Queued => {
                match next {
                    DefaultCredentialMappingStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultCredentialMappingStatus::Running => {
                match next {
                    DefaultCredentialMappingStatus::Done | DefaultCredentialMappingStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultCredentialMappingStatus::Done => {
                match next {
                    DefaultCredentialMappingStatus::Failed => {
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

    pub fn move_to(&mut self, next: DefaultCredentialMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default credential mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ratio + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultCredentialMapping {
        DefaultCredentialMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            summary: self.summary.clone(),
            retries: self.retries,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a default credential mapping in status new.
pub fn new_default_credential_mapping(code: &str, title: &str) -> DefaultCredentialMapping {
    DefaultCredentialMapping {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        summary: "".to_string(),
        retries: 0,
        country: "".to_string(),
        status: DefaultCredentialMappingStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_default_credential_mapping("code-942", "title-481");
        assert_eq!(item.status, DefaultCredentialMappingStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_default_credential_mapping("code-942", "title-481");
        if item.move_to(DefaultCredentialMappingStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultCredentialMappingStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_default_credential_mapping("code-942", "title-481");
        item.status = DefaultCredentialMappingStatus::Failed;
        assert!(
            item.move_to(DefaultCredentialMappingStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
