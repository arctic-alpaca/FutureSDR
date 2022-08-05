use gloo_worker::HandlerId;

#[derive(Debug)]
pub enum Msg<T> {
    Respond { output: T, id: HandlerId },
}
