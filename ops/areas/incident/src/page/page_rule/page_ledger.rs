use super::*;

use acme_kit::text;

/// Where a page ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PageLedgerStatus {
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

impl std::fmt::Display for PageLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a page ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PageLedger {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub namespace: String,
    pub country: String,
    pub revision: i64,
    pub unit: String,
    pub limit: i64,
    pub status: PageLedgerStatus,
}

impl PageLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageLedgerStatus) -> bool {
        match self.status {
            PageLedgerStatus::New => {
                match next {
                    PageLedgerStatus::Queued | PageLedgerStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageLedgerStatus::Queued => {
                match next {
                    PageLedgerStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageLedgerStatus::Running => {
                match next {
                    PageLedgerStatus::Done | PageLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageLedgerStatus::Done => {
                match next {
                    PageLedgerStatus::Failed => {
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

    pub fn move_to(&mut self, next: PageLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.revision as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageLedger {
        PageLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            namespace: self.namespace.clone(),
            country: self.country.clone(),
            revision: self.revision,
            unit: self.unit.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a page ledger in status new.
pub fn new_page_ledger(code: &str, title: &str) -> PageLedger {
    PageLedger {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        namespace: "".to_string(),
        country: "".to_string(),
        revision: 0,
        unit: "".to_string(),
        limit: 0,
        status: PageLedgerStatus::New,
    }
}
