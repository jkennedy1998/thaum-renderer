#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CellTexture {
    code: u8,
}

impl CellTexture {
    pub const fn none() -> Self {
        Self { code: 0 }
    }

    pub const fn new(code: u8) -> Self {
        Self { code }
    }

    pub const fn code(self) -> u8 {
        self.code
    }

    pub const fn is_none(self) -> bool {
        self.code == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_none_is_zero_code() {
        assert_eq!(CellTexture::none().code(), 0);
        assert!(CellTexture::none().is_none());
    }

    #[test]
    fn texture_code_round_trips() {
        let texture = CellTexture::new(3);
        assert_eq!(texture.code(), 3);
        assert!(!texture.is_none());
    }
}
