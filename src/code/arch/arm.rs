use crate::code::AssembleError;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelocationKind {
    None = 0,
    Jump24 = 29,
    Movw,
    Movt,
}

impl RelocationKind {
    /// Addend has already been applied to value
    pub(crate) fn apply_relative(self, data: &mut [u8], location: usize, value: usize) -> Result<(), AssembleError> {
        use RelocationKind::*;
        let value: isize = if location < value {
            (value - location).try_into().ok().ok_or(AssembleError::InvalidRelocation("Relative relocation difference too large"))?
        } else {
            -(location - value).try_into().ok().ok_or(AssembleError::InvalidRelocation("Relative relocation difference too large"))?
        };
        match self {
            None => Ok(()),
            Jump24 => {
                // Low 24 bits of (big-endian) word contain low 26 bits of pc-relative address (00 on the end) (sign-extended)
                let instruction: &mut [u8; 4] = data.get_mut(location..location+4)
                    .ok_or(AssembleError::InvalidRelocation("Attempted to apply relocation past end of section"))?
                    .try_into().unwrap();
                if value & 3 != 0{
                    return Err(AssembleError::InvalidRelocation("Relative relocation cut off low bits"));
                }
                let actual_value = value >> 2;
                let actual_value: i32 = actual_value.try_into().ok().ok_or(AssembleError::InvalidRelocation("Relative relocation difference too large"))?;
                let bytes = i32::to_le_bytes(actual_value);
                if !(
                    (bytes[3] == 0x00 && bytes[2] & 0x80 == 0x00) ||
                    (bytes[3] == 0xFF && bytes[2] & 0x80 == 0x80)
                ) {
                    return Err(AssembleError::InvalidRelocation("Relative relocation difference too large"));
                }
                instruction[..3].copy_from_slice(&bytes[..3]);
                Ok(())
            },
            Movw | Movt => Err(AssembleError::InvalidRelocation("Cannot apply direct relocation for relative symbol")),
        }
    }
    pub(crate) fn apply_absolute(self, data: &mut [u8], location: usize, value: isize) -> Result<(), AssembleError> {
        use RelocationKind::*;
        match self {
            None => Ok(()),
            Jump24 => Err(AssembleError::InvalidRelocation("Cannot apply relative relocation for absolute symbol")),
            Movw => {
                // Low 16 bits of value encoded in bits 19-16 and 11-0 of the 4-byte big-endian instruction
                let instruction: &mut [u8; 4] = data.get_mut(location..location+4)
                    .ok_or(AssembleError::InvalidRelocation("Attempted to apply relocation past end of section"))?
                    .try_into().unwrap();
                let actual_value: i32 = value.try_into().ok().ok_or(AssembleError::InvalidRelocation("Relative relocation difference too large"))?;
                let low_half: u16 = actual_value as u32 as u16;
                let imm4: u8 = (low_half >> 12) as u8 & 0xF;
                let imm12: u16 = low_half & 0xFFF;
                let imm12_bytes = imm12.to_be_bytes();
                instruction[2] &= 0xF0;
                instruction[2] |= imm4;
                instruction[1] &= 0xF0;
                instruction[1] |= imm12_bytes[0];
                instruction[0] = imm12_bytes[1];
                Ok(())
            },
            Movt => {
                // Low 16 bits of value encoded in bits 19-16 and 11-0 of the 4-byte big-endian instruction
                let instruction: &mut [u8; 4] = data.get_mut(location..location+4)
                    .ok_or(AssembleError::InvalidRelocation("Attempted to apply relocation past end of section"))?
                    .try_into().unwrap();
                let actual_value: i32 = value.try_into().ok().ok_or(AssembleError::InvalidRelocation("Relative relocation difference too large"))?;
                let high_half: u16 = (actual_value as u32 >> 16) as u16;
                let imm4: u8 = (high_half >> 12) as u8 & 0xF;
                let imm12: u16 = high_half & 0xFFF;
                let imm12_bytes = imm12.to_be_bytes();
                instruction[2] &= 0xF0;
                instruction[2] |= imm4;
                instruction[1] &= 0xF0;
                instruction[1] |= imm12_bytes[0];
                instruction[0] = imm12_bytes[1];
                Ok(())
            },
        }
    }
}