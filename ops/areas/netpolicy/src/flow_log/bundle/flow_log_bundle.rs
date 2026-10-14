use super::*;

use acme_kit::text;

/// Where a flow log bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogBundleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FlowLogBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FlowLogBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBundle {
    pub code: String,
    pub label: String,
    pub region: String,
    pub minimum: f64,
    pub channel: String,
    pub burst: i64,
    pub notes: String,
    pub phone: String,
    pub capacity: i64,
    pub key_id: String,
    pub unit: String,
    pub status: FlowLogBundleStatus,
}

impl FlowLogBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogBundleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogBundleStatus) -> bool {
        match self.status {
            FlowLogBundleStatus::Created => {
                match next {
                    FlowLogBundleStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogBundleStatus::Validated => {
                return next == FlowLogBundleStatus::Published;
            }
            FlowLogBundleStatus::Published => {
                match next {
                    FlowLogBundleStatus::Retired => {
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

    pub fn move_to(&mut self, next: FlowLogBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "flow log bundle", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.burst as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogBundle {
        FlowLogBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            minimum: self.minimum,
            channel: self.channel.clone(),
            burst: self.burst,
            notes: self.notes.clone(),
            phone: self.phone.clone(),
            capacity: self.capacity,
            key_id: self.key_id.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a flow log bundle in status created.
pub fn new_flow_log_bundle(code: &str, label: &str) -> FlowLogBundle {
    FlowLogBundle {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        minimum: 0.0,
        channel: "".to_string(),
        burst: 0,
        notes: "".to_string(),
        phone: "".to_string(),
        capacity: 0,
        key_id: "".to_string(),
        unit: "".to_string(),
        status: FlowLogBundleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_flow_log_bundle("code-70", "label-77");
        assert_eq!(item.status, FlowLogBundleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_flow_log_bundle("code-70", "label-77");
        if item.move_to(FlowLogBundleStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogBundleStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_flow_log_bundle("code-70", "label-77");
        item.status = FlowLogBundleStatus::Retired;
        assert!(
            item.move_to(FlowLogBundleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
