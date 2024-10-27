use errore::{span::SpanContext, subscriber::Subscriber, TraceAccess, TraceContext};

pub struct MyData {
    pub on_record: usize,
}

#[derive(Clone, Default, Debug)]
pub struct MySubscriber;

impl Subscriber for MySubscriber {
    fn on_start(&self, ctx: &mut SpanContext) {
        println!("[{}] on_start(): {}", ctx.record.target, ctx.record.name);

        // optionally any user data can be attached to the error-chain/trace-context
        ctx.extensions_mut().insert(MyData { on_record: 0 });
    }

    fn on_end(&self, ctx: &mut TraceContext) {
        println!("[{}] on_end(): {}", ctx.last().target, ctx.last().name);

        // access and print user data
        let target = ctx.last().target;
        let mut ext = ctx.extensions_mut();
        let data = ext.get_mut::<MyData>().expect("MyData must exist");

        println!("[{}] on_end(): traces={}", target, data.on_record);
    }

    fn on_new_span(&self, ctx: &mut SpanContext) {
        println!("[{}] on_new_span(): {}", ctx.record.target, ctx.record.name);
    }

    fn on_try_record(&self, ctx: &mut SpanContext) {
        println!(
            "[{}] on_try_record(): {}",
            ctx.record.target, ctx.record.name
        );
    }

    fn on_record(&self, ctx: &mut TraceContext) {
        println!("[{}] on_record(): {}", ctx.last().target, ctx.last().name);

        // access and modify user data
        let mut ext = ctx.extensions_mut();
        let data = ext.get_mut::<MyData>().expect("MyData must exist");
        data.on_record += 1;
    }
}

fn main() {
    errore::subscriber!(MySubscriber);
}
