use super::*;

use acme_kit::text;

/// Where a silence ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceLedgerStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SilenceLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SilenceLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a silence ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceLedger {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub summary: String,
    pub burst: i64,
    pub category: String,
    pub capacity: i64,
    pub archived: bool,
    pub starts_at: i64,
    pub status: SilenceLedgerStatus,
}

impl SilenceLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceLedgerStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceLedgerStatus) -> bool {
        match self.status {
            SilenceLedgerStatus::Created => {
                match next {
                    SilenceLedgerStatus::Published | SilenceLedgerStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceLedgerStatus::Validated => {
                match next {
                    SilenceLedgerStatus::Published | SilenceLedgerStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceLedgerStatus::Published => {
                match next {
                    SilenceLedgerStatus::Retired => {
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

    pub fn move_to(&mut self, next: SilenceLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.burst as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceLedger {
        SilenceLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            summary: self.summary.clone(),
            burst: self.burst,
            category: self.category.clone(),
            capacity: self.capacity,
            archived: self.archived,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a silence ledger in status created.
pub fn new_silence_ledger(code: &str, name: &str) -> SilenceLedger {
    SilenceLedger {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        summary: "".to_string(),
        burst: 0,
        category: "".to_string(),
        capacity: 0,
        archived: false,
        starts_at: 0,
        status: SilenceLedgerStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_silence_ledger("code-343", "name-984");
        assert_eq!(item.status, SilenceLedgerStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_silence_ledger("code-343", "name-984");
        if item.move_to(SilenceLedgerStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceLedgerStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_silence_ledger("code-343", "name-984");
        item.status = SilenceLedgerStatus::Retired;
        assert!(
            item.move_to(SilenceLedgerStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
