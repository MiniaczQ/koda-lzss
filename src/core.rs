use std::collections::VecDeque;

pub struct LzssCore<S> {
    dict: Vec<S>,
    buff: VecDeque<S>,
}

impl<S> LzssCore<S> {
    pub fn new(dict_size: usize, buff_size: usize) -> Self {
        let buff = VecDeque::with_capacity(buff_size);
        let dict = Vec::with_capacity(dict_size);
        Self { dict, buff }
    }
}
