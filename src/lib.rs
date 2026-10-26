mod ledger;

use tine_plugin_sdk::{notice, BlockFormat, Effect, Event};

fn handle(event: &Event) -> Result<Vec<Effect>, String> {
    match (event.kind.as_str(), event.contribution_id.as_deref()) {
        ("command", Some("readiness")) => Ok(vec![notice("Tine Ledger diagnostic alpha. Native capture, complete cross-page reports, and charts require additional host capabilities. No notes are changed.")]),
        ("command", Some("validate-current-block")) => {
            let block = event.focused_block.as_ref().ok_or("Focus one canonical Markdown ledger block first.")?;
            if block.format == Some(BlockFormat::Org) { return Err("This diagnostic validates canonical Markdown records only.".into()); }
            let record = ledger::validate_canonical_block(&block.raw)?;
            Ok(vec![notice(format!("Canonical CNY record: {} {}.{:02}. Read-only format check; links and graph-wide totals have not been verified.", record.kind, record.minor_units / 100, record.minor_units % 100))])
        }
        _ => Ok(Vec::new()),
    }
}

tine_plugin_sdk::tine_plugin!(handle);
