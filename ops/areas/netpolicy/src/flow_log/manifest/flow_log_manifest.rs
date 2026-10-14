use super::*;

use acme_kit::text;

/// Where a flow log manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogManifestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FlowLogManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for FlowLogManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log manifest as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogManifest {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub fingerprint: String,
    pub label: String,
    pub port: i64,
    pub sequence: i64,
    pub namespace: String,
    pub archived: bool,
    pub status: FlowLogManifestStatus,
}

impl FlowLogManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogManifestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogManifestStatus) -> bool {
        match self.status {
            FlowLogManifestStatus::Draft => {
                match next {
                    FlowLogManifestStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogManifestStatus::Pending => {
                return next == FlowLogManifestStatus::Suspended;
            }
            FlowLogManifestStatus::Active => {
                return next == FlowLogManifestStatus::Closed;
            }
            FlowLogManifestStatus::Suspended => {
                match next {
                    FlowLogManifestStatus::Closed => {
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

    pub fn move_to(&mut self, next: FlowLogManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.sequence as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogManifest {
        FlowLogManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            fingerprint: self.fingerprint.clone(),
            label: self.label.clone(),
            port: self.port,
            sequence: self.sequence,
            namespace: self.namespace.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a flow log manifest in status draft.
pub fn new_flow_log_manifest(code: &str, name: &str) -> FlowLogManifest {
    FlowLogManifest {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        fingerprint: "".to_string(),
        label: "".to_string(),
        port: 0,
        sequence: 0,
        namespace: "".to_string(),
        archived: false,
        status: FlowLogManifestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_flow_log_manifest("code-427", "name-892");
        assert_eq!(item.status, FlowLogManifestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_flow_log_manifest("code-427", "name-892");
        if item.move_to(FlowLogManifestStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogManifestStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_flow_log_manifest("code-427", "name-892");
        item.status = FlowLogManifestStatus::Closed;
        assert!(
            item.move_to(FlowLogManifestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
