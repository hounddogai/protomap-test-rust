use super::*;

use acme_kit::text;

/// Where a regional verifier summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalVerifierSummaryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalVerifierSummaryStatus {
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

impl std::fmt::Display for RegionalVerifierSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional verifier summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierSummary {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub score: f64,
    pub cipher: String,
    pub priority: i64,
    pub title: String,
    pub status: RegionalVerifierSummaryStatus,
}

impl RegionalVerifierSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalVerifierSummaryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalVerifierSummaryStatus) -> bool {
        match self.status {
            RegionalVerifierSummaryStatus::New => {
                return next == RegionalVerifierSummaryStatus::Failed;
            }
            RegionalVerifierSummaryStatus::Queued => {
                match next {
                    RegionalVerifierSummaryStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalVerifierSummaryStatus::Running => {
                return next == RegionalVerifierSummaryStatus::Done || next == RegionalVerifierSummaryStatus::Failed;
            }
            RegionalVerifierSummaryStatus::Done => {
                match next {
                    RegionalVerifierSummaryStatus::Failed => {
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

    pub fn move_to(&mut self, next: RegionalVerifierSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional verifier summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional verifier summary", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.priority as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalVerifierSummary {
        RegionalVerifierSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            score: self.score,
            cipher: self.cipher.clone(),
            priority: self.priority,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional verifier summary in status new.
pub fn new_regional_verifier_summary(code: &str, name: &str) -> RegionalVerifierSummary {
    RegionalVerifierSummary {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        score: 0.0,
        cipher: "".to_string(),
        priority: 0,
        title: "".to_string(),
        status: RegionalVerifierSummaryStatus::New,
    }
}
