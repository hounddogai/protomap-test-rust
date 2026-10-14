use super::*;

use acme_kit::text;

/// Where a query bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl QueryBundleStatus {
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

impl std::fmt::Display for QueryBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One query bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryBundle {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub rate: f64,
    pub starts_at: i64,
    pub visible: bool,
    pub ceiling: f64,
    pub duration_minutes: i64,
    pub burst: i64,
    pub status: QueryBundleStatus,
}

impl QueryBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryBundleStatus) -> bool {
        match self.status {
            QueryBundleStatus::New => {
                match next {
                    QueryBundleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryBundleStatus::Queued => {
                match next {
                    QueryBundleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryBundleStatus::Running => {
                return next == QueryBundleStatus::Done || next == QueryBundleStatus::Failed;
            }
            QueryBundleStatus::Done => {
                match next {
                    QueryBundleStatus::Failed => {
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

    pub fn move_to(&mut self, next: QueryBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query bundle {} cannot move from {} to {}",
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
        format!("{} {}", "query bundle", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.cost * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryBundle {
        QueryBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            rate: self.rate,
            starts_at: self.starts_at,
            visible: self.visible,
            ceiling: self.ceiling,
            duration_minutes: self.duration_minutes,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a query bundle in status new.
pub fn new_query_bundle(code: &str, name: &str) -> QueryBundle {
    QueryBundle {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        rate: 0.0,
        starts_at: 0,
        visible: false,
        ceiling: 0.0,
        duration_minutes: 0,
        burst: 0,
        status: QueryBundleStatus::New,
    }
}
