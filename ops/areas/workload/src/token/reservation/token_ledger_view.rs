use super::*;

/// How screens and exports show a token ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenLedgerView {
    pub code: String,
    pub name: String,
    pub sequence: String,
    pub capacity: String,
    pub ratio: String,
    pub batch_size: String,
    pub cluster_text: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_token_ledger_view(item: &TokenLedger) -> TokenLedgerView {
    TokenLedgerView {
        code: item.code.clone(),
        name: item.name.clone(),
        sequence: item.sequence.to_string(),
        capacity: item.capacity.to_string(),
        ratio: format!("{:.2}", item.ratio),
        batch_size: item.batch_size.to_string(),
        cluster_text: item.cluster.clone(),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_ledger_views(items: &[TokenLedger]) -> Vec<TokenLedgerView> {
    let mut views: Vec<TokenLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_ledger_view(item));
    }
    views
}

pub fn sum_token_ledger_view(view: &TokenLedgerView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_capacity as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_ledger("code-376", "name-262");
        let view = to_token_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
