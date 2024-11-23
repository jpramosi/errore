use std::ops::{Deref, DerefMut};
use std::sync::{LazyLock, Mutex};

use errore::prelude::*;
use errore::span::SpanContext;
use errore::subscriber::Subscriber;

#[derive(Clone, Debug, Default)]
pub struct TestContextData {
    pub buffer: String,
    pub on_new_span: usize,
    pub on_try_record: usize,
    pub on_record: usize,
    pub on_start: usize,
    pub on_end: usize,
}

impl Deref for TestContextData {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for TestContextData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

pub static DATA: LazyLock<Mutex<Option<TestContextData>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Clone, Debug, Default)]
pub struct TestSubscriber;

impl Subscriber for TestSubscriber {
    fn on_start(&self, builder: &mut TraceContextBuilder, rec: &TraceRecord) {
        let mut data = TestContextData::default();

        data.push_str(&format!("on_start(): {}\n", rec));
        data.on_start += 1;

        builder.extensions_mut().insert(data);
    }

    fn on_end(&self, ctx: &mut TraceContext) {
        let msg = format!("on_end(): {}\n", ctx.last());
        let mut ext = ctx.extensions_mut();

        let mut data = ext
            .remove::<TestContextData>()
            .expect("TestContextData should exist");

        data.push_str(&msg);
        data.on_end += 1;
        *DATA.lock().unwrap() = Some(data);
    }

    fn on_new_span(&self, ctx: &mut SpanContext) {
        let record = ctx.record;
        let mut ext = ctx.extensions_mut();

        let data = ext
            .get_mut::<TestContextData>()
            .expect("TestContextData should exist");

        data.push_str(&format!("on_new_span(): {}\n", record));
        data.on_new_span += 1;
    }

    fn on_try_record(&self, ctx: &mut SpanContext) {
        let record = ctx.record;
        let mut ext = ctx.extensions_mut();

        let data = ext
            .get_mut::<TestContextData>()
            .expect("TestContextData should exist");

        data.push_str(&format!("on_try_record(): {}\n", record));
        data.on_try_record += 1;
    }

    fn on_record(&self, ctx: &mut TraceContext) {
        let msg = format!("on_record(): {}\n", ctx.last());
        let mut ext = ctx.extensions_mut();

        let data = ext
            .get_mut::<TestContextData>()
            .expect("TestContextData should exist");

        data.push_str(&msg);
        data.on_record += 1;
    }
}

#[doc(hidden)]
pub fn format_diff(chunks: Vec<dissimilar::Chunk>) -> String {
    let mut buf = String::new();
    for chunk in chunks {
        let formatted = match chunk {
            dissimilar::Chunk::Equal(text) => text.into(),
            dissimilar::Chunk::Delete(text) => format!("\x1b[41m{}\x1b[0m", text),
            dissimilar::Chunk::Insert(text) => format!("\x1b[42m{}\x1b[0m", text),
        };
        buf.push_str(&formatted);
    }
    buf
}

#[doc(hidden)]
#[macro_export]
macro_rules! assert_eq_text {
    ($left:expr, $right:expr) => {
        let left = $left;
        let left = left.trim();
        let right = $right;
        let right = right.trim();
        if left != right {
            let diff = dissimilar::diff(left, right);
            panic!(
                "assertion `left == right` failed\n\n#### LEFT ####\n{}\n\n#### RIGHT ####\n{}\n\n#### DIFF ####\n{}\n",
                left,
                right,
                format_diff(diff)
            );
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! assert_eq_file {
    ($( $text:expr ),*) => {
        let module_name = module_path!().rsplit("::").next().unwrap();
        let dir = std::path::PathBuf::from(file!()).parent().unwrap().to_path_buf();
        let path = dir.join(format!("{}/{}.stdout", module_name, function!()));

        let mut left = String::with_capacity(1024);
        $(
            left.push_str($text.trim());
            left.push_str("\n\n");
        )*
        let left = left.trim();

        if std::option_env!("OVERWRITE").is_some() {
            let _ = std::fs::create_dir_all(&path.parent().unwrap());
            std::fs::write(&path, left).expect(&format!("Failed to write to {:?}", path));
        }
        else {
            if !path.exists() {
                panic!("Test file {:?} doesn't exist", path);
            }
            let content = std::fs::read_to_string(&path).expect(&format!("Failed to read from {:?}", path));

            let right = content.trim();
            if left != right {
                // miri is shitting the bed with dissimilar
                if let std::result::Result::Ok(_) = std::env::var("MIRIFLAGS") {
                    panic!(
                        "assertion `left == right` failed\n\n#### LEFT ####\n{}\n\n#### RIGHT ####\n{}\n",
                        left,
                        right
                    );
                }
                else {
                    let diff = dissimilar::diff(&left, right);
                    panic!(
                        "assertion `left == right` failed\n\n#### LEFT ####\n{}\n\n#### RIGHT ####\n{}\n\n#### DIFF ####\n{}\n",
                        left,
                        right,
                        format_diff(diff)
                    );
                }
            }
        }
    };
}
