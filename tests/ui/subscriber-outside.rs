use errore::subscriber::Subscriber;

#[derive(Clone, Default, Debug)]
pub struct MySubscriber;

impl Subscriber for MySubscriber {}

errore::subscriber!(MySubscriber);

fn main() {}
