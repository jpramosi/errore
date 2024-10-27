use errore::{subscriber::Subscriber, Extract, Id, TraceContext};
use tracing::{info, warn};

use crate::account;

#[derive(Clone, Default, Debug)]
pub struct TracingSubscriber;

impl Subscriber for TracingSubscriber {
    fn on_end(&self, ctx: &mut TraceContext) {
        let rec = ctx.last();

        // filter by crate
        if rec.target_id != Id::from_target("example_axum") {
            return;
        }

        // 'ctx.get::<account::Error>()' can also be used if the value is needed
        if ctx.has::<account::Error>() {
            // print a more detailed error report with a backtrace
            warn!("{}", ctx);
        } else {
            // print simple one-line error with name and location
            info!("{}", rec);
        }
    }
}
