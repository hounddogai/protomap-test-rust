use super::*;

use acme_kit::text;

/// Where a page rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PageRuleStatus {
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

impl std::fmt::Display for PageRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page rule as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageRule {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub burst: i64,
    pub score: f64,
    pub rate: f64,
    pub expires_at: i64,
    pub contact_email: String,
    pub status: PageRuleStatus,
}

impl PageRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageRuleStatus) -> bool {
        match self.status {
            PageRuleStatus::New => {
                match next {
                    PageRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageRuleStatus::Queued => {
                match next {
                    PageRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageRuleStatus::Running => {
                match next {
                    PageRuleStatus::Done | PageRuleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageRuleStatus::Done => {
                return next == PageRuleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PageRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.burst as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageRule {
        PageRule {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            burst: self.burst,
            score: self.score,
            rate: self.rate,
            expires_at: self.expires_at,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a page rule in status new.
pub fn new_page_rule(code: &str, title: &str) -> PageRule {
    PageRule {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        burst: 0,
        score: 0.0,
        rate: 0.0,
        expires_at: 0,
        contact_email: "".to_string(),
        status: PageRuleStatus::New,
    }
}
