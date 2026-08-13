#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CellWarble {
    code: u8,
}

impl CellWarble {
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
    fn warble_none_is_zero_code() {
        assert_eq!(CellWarble::none().code(), 0);
        assert!(CellWarble::none().is_none());
    }

    #[test]
    fn warble_code_round_trips() {
        let warble = CellWarble::new(5);
        assert_eq!(warble.code(), 5);
        assert!(!warble.is_none());
    }
}
