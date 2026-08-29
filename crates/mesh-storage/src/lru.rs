pub struct LruMempool { max_size: usize }
impl LruMempool { pub fn new() -> Self { Self { max_size: 1000 } } }
