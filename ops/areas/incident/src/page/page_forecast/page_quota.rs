use super::*;

use acme_kit::text;

/// Where a page quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PageQuotaStatus {
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

impl std::fmt::Display for PageQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One page quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageQuota {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub summary: String,
    pub host: String,
    pub status: PageQuotaStatus,
}

impl PageQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageQuotaStatus) -> bool {
        match self.status {
            PageQuotaStatus::New => {
                match next {
                    PageQuotaStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageQuotaStatus::Queued => {
                match next {
                    PageQuotaStatus::Done | PageQuotaStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageQuotaStatus::Running => {
                return next == PageQuotaStatus::Done || next == PageQuotaStatus::Failed;
            }
            PageQuotaStatus::Done => {
                return next == PageQuotaStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PageQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.ratio
    }

    pub fn copy(&self) -> PageQuota {
        PageQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            summary: self.summary.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a page quota in status new.
pub fn new_page_quota(code: &str, title: &str) -> PageQuota {
    PageQuota {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        summary: "".to_string(),
        host: "".to_string(),
        status: PageQuotaStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_page_quota("code-893", "title-453");
        assert_eq!(item.status, PageQuotaStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_page_quota("code-893", "title-453");
        if item.move_to(PageQuotaStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageQuotaStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_page_quota("code-893", "title-453");
        item.status = PageQuotaStatus::Failed;
        assert!(item.move_to(PageQuotaStatus::New).is_err(), "expected move to fail");
    }
}
