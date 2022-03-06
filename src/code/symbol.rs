use std::sync::atomic::AtomicUsize;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SymbolInner {
    Local(usize),
    Global(String),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol {
    inner: SymbolInner,
}

static LOCAL_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl Symbol {
    pub fn new_global(sym: String) -> Self {
        Symbol { inner: SymbolInner::Global(sym) }
    }
    pub fn new_local() -> Self {
        let idx = LOCAL_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Symbol { inner: SymbolInner::Local(idx) }
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SymbolInner::*;
        match &self.inner {
            Local(idx) => write!(f, "\".L{}\"", idx),
            Global(sym) => write!(f, "{:?}", sym),
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SymbolInner::*;
        match &self.inner {
            Local(idx) => write!(f, ".L{}", idx),
            Global(sym) => write!(f, "{}", sym),
        }
    }
}
