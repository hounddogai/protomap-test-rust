use super::*;

use acme_kit::text;

/// Where a nonce threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceThresholdStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl NonceThresholdStatus {
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

impl std::fmt::Display for NonceThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce threshold as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceThreshold {
    pub code: String,
    pub name: String,
    pub title: String,
    pub discount: f64,
    pub tags: Vec<String>,
    pub principal: String,
    pub status: NonceThresholdStatus,
}

impl NonceThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceThresholdStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceThresholdStatus) -> bool {
        match self.status {
            NonceThresholdStatus::New => {
                match next {
                    NonceThresholdStatus::Queued | NonceThresholdStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceThresholdStatus::Queued => {
                match next {
                    NonceThresholdStatus::Done | NonceThresholdStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceThresholdStatus::Running => {
                return next == NonceThresholdStatus::Done || next == NonceThresholdStatus::Failed;
            }
            NonceThresholdStatus::Done => {
                match next {
                    NonceThresholdStatus::Failed => {
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

    pub fn move_to(&mut self, next: NonceThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "nonce threshold", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> NonceThreshold {
        NonceThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            discount: self.discount,
            tags: self.tags.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a nonce threshold in status new.
pub fn new_nonce_threshold(code: &str, name: &str) -> NonceThreshold {
    NonceThreshold {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        discount: 0.0,
        tags: Vec::new(),
        principal: "".to_string(),
        status: NonceThresholdStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_nonce_threshold("code-616", "name-172");
        assert_eq!(item.status, NonceThresholdStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_nonce_threshold("code-616", "name-172");
        if item.move_to(NonceThresholdStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceThresholdStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_nonce_threshold("code-616", "name-172");
        item.status = NonceThresholdStatus::Failed;
        assert!(
            item.move_to(NonceThresholdStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
