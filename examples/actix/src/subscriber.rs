use errore::{subscriber::Subscriber, Extract, Id, TraceContext};
use log::{info, warn};

use crate::account;

#[derive(Clone, Default, Debug)]
pub struct ErrorSubscriber;

impl Subscriber for ErrorSubscriber {
    fn on_end(&self, ctx: &mut TraceContext) {
        let rec = ctx.last();

        // filter by crate
        if rec.target_id != Id::from_target("example_actix") {
            return;
        }

        // 'ctx.get::<account::Error>()' can also be used if the value is needed
        if ctx.has::<account::Error>() {
            // print simple one-line error with name and location
            info!("{}", rec);
        } else {
            // print a more detailed error report with a backtrace
            warn!("{}", ctx);
        }
    }
}
