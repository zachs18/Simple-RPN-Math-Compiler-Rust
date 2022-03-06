use crate::code::AssembleError;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelocationKind {
    None = 0,
    Direct32 = 1,
    Pc32 = 2,
}

impl RelocationKind {
    /// Addend has already been applied to value
    pub(crate) fn apply_relative(self, data: &mut [u8], location: usize, value: usize) -> Result<(), AssembleError> {
        use RelocationKind::*;
        match self {
            None => Ok(()),
            Pc32 => {
                let reloc_slice: &mut [u8; 4] = data.get_mut(location..location+4)
                    .ok_or(AssembleError::InvalidRelocation("Attempted to apply relocation past end of section"))?
                    .try_into().unwrap();
                let value: isize = if location < value {
                    (value - location).try_into().ok().ok_or(AssembleError::InvalidRelocation("Relative relocation difference too large"))?
                } else {
                    -(location - value).try_into().ok().ok_or(AssembleError::InvalidRelocation("Relative relocation difference too large"))?
                };
                let actual_value: i32 = value.try_into().ok().ok_or(AssembleError::InvalidRelocation("Relative relocation difference too large"))?;
                *reloc_slice = i32::to_ne_bytes(actual_value);
                Ok(())
            },
            Direct32 => Err(AssembleError::InvalidRelocation("Cannot apply Direct32 relocation for relative symbol")),
        }
    }
    pub(crate) fn apply_absolute(self, data: &mut [u8], location: usize, value: isize) -> Result<(), AssembleError> {
        todo!()
    }
}