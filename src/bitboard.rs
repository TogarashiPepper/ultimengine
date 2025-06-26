use crate::{
    board::{Slot, State},
    generated::{ONE_AWAY_O, ONE_AWAY_X, WON_BY_O, WON_BY_X},
};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "savestates", derive(Encode, Decode))]
/// u32 holding 3 x 9 variants + 5 bits for state
/// First 9 bits are X, then O, then Empty, then 5 bits for state
pub struct BitBoard(pub u32);

impl BitBoard {
    pub const fn new() -> Self {
        const { Self::new_with([Slot::Empty; 9]) }
    }

    pub const fn new_with(brd: [Slot; 9]) -> Self {
        let mut bits = 0;
        let mut idx = 0;

        loop {
            if idx == 9 {
                break BitBoard(bits);
            }

            bits |= match brd[idx] {
                Slot::X => 1 << idx,
                Slot::O => 1 << (9 + idx),
                Slot::Empty => 1 << (18 + idx),
                // no-op for disabled
                Slot::Disabled => 0,
            };

            idx += 1;
        }
    }

    pub const fn to_arr(self) -> [Slot; 9] {
        let mut buf = [Slot::Disabled; 9];
        let mut idx = 0;

        loop {
            if idx == 9 {
                break buf;
            }

            if self.0 & (1 << idx) == (1 << idx) {
                buf[idx] = Slot::X;
            } else if self.0 & (1 << (9 + idx)) == (1 << (9 + idx)) {
                buf[idx] = Slot::O;
            } else if self.0 & (1 << (18 + idx)) == (1 << (18 + idx)) {
                buf[idx] = Slot::Empty;
            }

            idx += 1;
        }
    }

    pub fn to_3x3(brd: [Slot; 9]) -> String {
        use std::fmt::Write;

        let mut s = String::new();

        writeln!(s, "{} {} {}", brd[0], brd[1], brd[2]).unwrap();
        writeln!(s, "{} {} {}", brd[3], brd[4], brd[5]).unwrap();
        writeln!(s, "{} {} {}", brd[6], brd[7], brd[8]).unwrap();

        s
    }

    pub const fn flip(&mut self) {
        const MASK: u32 = 2u32.pow(9) - 1;
        const MASK2: u32 = 2u32.pow(18) - 1;

        let xs = self.0 & MASK;
        let os = self.0 & MASK << 9;

        self.0 &= !MASK2;
        self.0 |= xs << 9;
        self.0 |= os >> 9;

        self.set_state(self.state().flip());
    }

    pub const fn state(self) -> State {
        State::from_u32(self.0 >> 27)
    }

    pub const fn set_state(&mut self, st: State) {
        const MASK: u32 = 0b11111 << 27;
        self.0 &= !MASK;

        self.0 |= st.to_u32() << 27;
    }

    pub const fn corners(self, side: Slot) -> i32 {
        const CORNER_MASK: u32 = 0b101000101;

        match side {
            Slot::X => (self.0 & CORNER_MASK).count_ones() as i32,
            Slot::O => (self.0 & (CORNER_MASK << 9)).count_ones() as i32,

            _ => unreachable!(),
        }
    }
    
    #[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
    const fn one_aways<const FOR_X: bool>(self) -> i32 {
        let mut n = 0;
        let mut idx = 0;
        let arr = const {
            if FOR_X {
                ONE_AWAY_X
            }
            else {
                ONE_AWAY_O
            }
        };

        loop {
            if idx >= 24 {
                break;
            }

            let masks = arr[idx];

            n += (b == (b & self.0)) as i32;

            idx += 8;
        }

        n
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    fn one_aways<const FOR_X: bool>(self) -> i32 {
        use std::arch::aarch64::{
            vaddvq_u32, vandq_u32, vceqq_u32, vld1q_dup_u32, vld1q_u32_x3, vmvnq_u32, vshrq_n_u32
        };

        let mut n = 0;
        let mut idx = 0;
        let arr = const { if FOR_X { ONE_AWAY_X } else { ONE_AWAY_O } };
        let brd = unsafe { vld1q_dup_u32(&self.0 as *const u32) };

        let zero: u32 = 0;
        let zeros = unsafe { vld1q_dup_u32(&zero as *const u32) };

        loop {
            if idx >= 24 {
                break;
            }

            let r = unsafe {
                let masks = vld1q_u32_x3(arr.as_ptr().add(idx));

                let and0 = vandq_u32(masks.0, brd);
                let and1 = vandq_u32(masks.1, brd);
                let and2 = vandq_u32(masks.2, brd);

                let eq0 = vceqq_u32(and0, masks.0);
                let eq1 = vceqq_u32(and1, masks.1);
                let eq2 = vceqq_u32(and2, masks.2);

                let eq0 = vceqq_u32(vmvnq_u32(eq0), zeros);
                let eq1 = vceqq_u32(vmvnq_u32(eq1), zeros);
                let eq2 = vceqq_u32(vmvnq_u32(eq2), zeros);

                let shf0 = vshrq_n_u32::<31>(eq0);
                let shf1 = vshrq_n_u32::<31>(eq1);
                let shf2 = vshrq_n_u32::<31>(eq2);

                let cnt0 = vaddvq_u32(shf0);
                let cnt1 = vaddvq_u32(shf1);
                let cnt2 = vaddvq_u32(shf2);

                cnt0 + cnt1 + cnt2
            };

            n += r as i32;

            idx += 12;
        }

        n
    }

    pub fn one_aways_x(self) -> i32 {
        self.one_aways::<true>()
    }

    pub fn one_aways_o(self) -> i32 {
        self.one_aways::<false>()
    }

    const fn won_by<const FOR_X: bool>(self) -> bool {
        let mut idx = 0;
        let arr = const { if FOR_X { WON_BY_X } else { WON_BY_O } };

        loop {
            if idx == 8 {
                break false;
            }

            let b = arr[idx];

            if b == (b & self.0) {
                break true;
            }

            idx += 1;
        }
    }

    pub const fn won_by_x(self) -> bool {
        self.won_by::<true>()
    }

    pub const fn won_by_o(self) -> bool {
        self.won_by::<false>()
    }
}

impl Default for BitBoard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::BitBoard;
    use crate::board::{
        Slot::{Empty as E, *},
        State,
    };

    #[test]
    fn set_state() {
        let mut brd = BitBoard::new();
        assert_eq!(brd.state(), State::Undecided);

        brd.set_state(State::Won);
        assert_eq!(brd.state(), State::Won);

        brd.set_state(State::Tied);
        assert_eq!(brd.state(), State::Tied);
    }

    #[test]
    fn flip() {
        let mut brd = BitBoard::new_with([X, X, O, E, E, E, O, O, X]);
        brd.flip();

        assert_eq!(brd, BitBoard::new_with([O, O, X, E, E, E, X, X, O]));
    }

    #[test]
    fn to_arr() {
        let brd = BitBoard::new_with([X, E, X, X, X, O, O, O, O]);

        assert_eq!(BitBoard::new_with(brd.to_arr()), brd);
    }

    #[test]
    fn won_byx() {
        let brd = BitBoard::new_with([X, X, X, E, E, E, E, E, E]);
        assert!(brd.won_by_x());

        let brd = BitBoard::new_with([X, E, X, E, X, E, E, E, X]);
        assert!(brd.won_by_x());
    }

    #[test]
    fn won_byo() {
        let brd = BitBoard::new_with([O, O, O, E, E, E, E, E, E]);
        assert!(brd.won_by_o());

        let brd = BitBoard::new_with([O, E, O, E, O, E, E, E, O]);
        assert!(brd.won_by_o());
    }

    #[test]
    fn one_aways_x() {
        let brd = BitBoard::new_with([E, E, E, X, E, X, E, E, X]);
        assert_eq!(brd.one_aways_x(), 2);

        let brd = BitBoard::new_with([E; 9]);
        assert_eq!(brd.one_aways_x(), 0);

        let brd = BitBoard::new_with([X; 9]);
        assert_eq!(brd.one_aways_x(), 0);

        let brd = BitBoard::new_with([X, X, E, X, X, E, E, E, E]);
        assert_eq!(brd.one_aways_x(), 5);
    }

    #[test]
    fn one_aways_o() {
        let brd = BitBoard::new_with([E, E, E, O, E, O, E, E, O]);
        assert_eq!(brd.one_aways_o(), 2);

        let brd = BitBoard::new_with([E; 9]);
        assert_eq!(brd.one_aways_o(), 0);

        let brd = BitBoard::new_with([O; 9]);
        assert_eq!(brd.one_aways_o(), 0);

        let brd = BitBoard::new_with([O, O, E, O, O, E, E, E, E]);
        assert_eq!(brd.one_aways_o(), 5);
    }

    #[test]
    fn state_won() {
        let mut brd = BitBoard::new();
        brd.0 |= 1 << 27;

        assert_eq!(brd.state(), State::Won);
    }

    #[test]
    fn state_lost() {
        let mut brd = BitBoard::new();
        brd.0 |= 2 << 27;

        assert_eq!(brd.state(), State::Lost);
    }

    #[test]
    fn state_tied() {
        let mut brd = BitBoard::new();
        brd.0 |= 3 << 27;

        assert_eq!(brd.state(), State::Tied);
    }

    #[test]
    fn state_undecided() {
        let mut brd = BitBoard::new();
        brd.0 |= 0 << 27;

        assert_eq!(brd.state(), State::Undecided);
    }

    #[test]
    fn corners() {
        let brd = BitBoard::new_with([X, O, X, O, O, O, X, O, O]);

        assert_eq!(brd.corners(X), 3);
        assert_eq!(brd.corners(O), 1);
    }
}
