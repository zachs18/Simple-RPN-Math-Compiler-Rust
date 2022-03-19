use std::{borrow::Cow, collections::HashMap};

pub mod arch;
pub(crate) use arch::RelocationKind;

mod symbol;
pub(crate) use symbol::Symbol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relocation {
    location: usize,
    kind: arch::RelocationKind,
    symbol: Symbol,
    addend: isize,
}

impl Relocation {
    pub(crate) fn new(location: usize, kind: arch::RelocationKind, symbol: Symbol, addend: isize) -> Self { Self { location, kind, symbol, addend } }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Relocatable {
    /// Raw data
    pub(crate) data: Cow<'static, [u8]>,
    /// Power of two minimum alignment of this data (0 means 1, 1 means 2, 2 means 4, etc.)
    pub(crate) alignment: u32,
    /// Vector of symbol definitions in this section of data
    pub(crate) symbols: Vec<(Symbol, usize)>,
    /// Vector of absolute symbol definitions
    pub(crate) abs_symbols: Vec<(Symbol, isize)>,
    /// Vector of relocations to be applied to this section of data
    pub(crate) relocations: Vec<Relocation>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Object {
    pub code: Relocatable,
    pub data: Relocatable,
}

impl From<Relocatable> for Object {
    fn from(code: Relocatable) -> Self {
        Self { code, data: Relocatable::default() }
    }
}

impl std::ops::Add for Object {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl std::ops::AddAssign for Object {
    fn add_assign(&mut self, rhs: Self) {
        self.code += rhs.code;
        self.data += rhs.data;
    }
}

macro_rules! impl_from_data_for_relocatable {
    ($ty:ty) => {
        impl From<$ty> for Relocatable {
            fn from(data: $ty) -> Self {
                Self {
                    data: data.into(),
                    alignment: 0,
                    symbols: vec![],
                    abs_symbols: vec![],
                    relocations: vec![],
                }
            }
        }
        impl From<$ty> for Object {
            fn from(code: $ty) -> Self {
                Self::from(Relocatable::from(code))
            }
        }
    }
}

impl_from_data_for_relocatable!(&'static [u8]);
impl_from_data_for_relocatable!(Cow<'static, [u8]>);
impl_from_data_for_relocatable!(Vec<u8>);


impl<const N: usize> From<[u8; N]> for Relocatable {
    fn from(data: [u8; N]) -> Self {
        Self {
            data: Vec::from(Box::new(data) as Box<[u8]>).into(),
            alignment: 0,
            symbols: vec![],
            abs_symbols: vec![],
            relocations: vec![],
        }
    }
}
impl<const N: usize> From<[u8; N]> for Object {
    fn from(code: [u8; N]) -> Self {
        Self::from(Relocatable::from(code))
    }
}

impl std::ops::Add for Relocatable {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl std::ops::AddAssign for Relocatable {
    fn add_assign(&mut self, rhs: Self) {
        {
            // Pad self so rhs is still correctly aligned
            let rhs_align = 1usize << rhs.alignment;
            let padding = (rhs_align - (self.data.len() % rhs_align)) % rhs_align;
            if padding > 0 {
                self.data.to_mut().reserve(padding + rhs.data.len());
                let new_lhs_len = self.data.len() + padding;
                self.data.to_mut().resize(new_lhs_len, 0);
            }
        }

        let lhs_len = self.data.len();
        if rhs.data.len() > 0 {
            self.data.to_mut().extend_from_slice(&rhs.data);
        }

        self.alignment = self.alignment.max(rhs.alignment);

        self.abs_symbols.extend(rhs.abs_symbols.into_iter().map(
            |(sym, val)| (sym, val)
        ));

        self.symbols.extend(rhs.symbols.into_iter().map(
            |(sym, loc)| (sym, loc + lhs_len)
        ));

        self.relocations.extend(rhs.relocations.into_iter().map(
            |Relocation { location, kind, symbol, addend }|
                Relocation { location: location + lhs_len, kind, symbol, addend}
            // |(loc, reloc, sym)| (loc + lhs_len, reloc, sym)
        ));
    }
}

#[derive(Debug, Clone)]
pub enum AssembleError {
    UndefinedSymbol(Symbol),
    MultiplyDefinedSymbol(Symbol),
    InvalidRelocation(&'static str)
}

impl Relocatable {
    pub fn assemble(&self) -> Result<Vec<u8>, AssembleError> {
        enum Value {
            Relative(usize),
            Absolute(isize),
        }

        let mut data = (*self.data).to_owned();

        let mut symbols: HashMap<&Symbol, Value> = HashMap::new();
        for (sym, loc) in &self.symbols {
            match symbols.insert(sym, Value::Relative(*loc)) {
                Some(_) => {
                    return Err(AssembleError::MultiplyDefinedSymbol(sym.clone()));
                },
                None => {},
            }
        }

        for (sym, val) in &self.abs_symbols {
            match symbols.insert(sym, Value::Absolute(*val)) {
                Some(_) => {
                    return Err(AssembleError::MultiplyDefinedSymbol(sym.clone()));
                },
                None => {},
            }
        }

        for Relocation { location, kind, symbol, addend } in &self.relocations {
            match symbols.get(symbol) {
                Some(Value::Relative(val)) => {
                    let val = i128::try_from(*val).unwrap();
                    let val = val.checked_add((*addend).try_into().unwrap()).unwrap();
                    let val = val.try_into()
                        .ok().ok_or(AssembleError::InvalidRelocation("Addend placed value outside of range"))?;
                    kind.apply_relative(&mut data[..], *location, val)?;
                },
                Some(Value::Absolute(val)) => {
                    let val = val.checked_add(*addend)
                        .ok_or(AssembleError::InvalidRelocation("Addend placed value outside of range"))?;
                    kind.apply_absolute(&mut data[..], *location, val)?;
                },
                None => {
                    return Err(AssembleError::UndefinedSymbol(symbol.clone()));
                }
            };
        }
        Ok(data)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
