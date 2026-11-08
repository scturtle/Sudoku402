//! Browser-independent Sudoku model. Candidate bit `1 << digit` matches the
//! representation observed in Sudoku 401; this is not a compatibility layer.
const ALL: u16 = 0x3fe;
mod logic;
pub use logic::{
    EliminationHint, LogicHint, ALL_TECHNIQUES, ALMOST_LOCKED_SETS, ALS_CHAIN, FINNED_JELLYFISH,
    FINNED_SQUIRMBAG, FINNED_SWORDFISH, FINNED_X_WING, FULL_HOUSE, HIDDEN_PAIR, HIDDEN_QUAD,
    HIDDEN_SINGLE, HIDDEN_TRIPLE, JELLYFISH, LOCKED_CANDIDATES, NAKED_PAIR, NAKED_QUAD,
    NAKED_SINGLE, NAKED_TRIPLE, SQUIRMBAG, SWORDFISH, WXYZ_WING, XYZ_WING, XY_WING, X_WING,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cell {
    pub given: u8,
    pub value: u8,
    pub candidates: u16,
}

impl Cell {
    pub fn digit(self) -> u8 {
        if self.given != 0 {
            self.given
        } else {
            self.value
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    cells: [Cell; 81],
    auto_candidates: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hint {
    pub index: usize,
    pub digit: u8,
    pub technique: &'static str,
    pub unit: Option<Unit>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Row(usize),
    Column(usize),
    Block(usize),
}

impl Unit {
    pub fn cells(self) -> [usize; 9] {
        std::array::from_fn(|i| match self {
            Self::Row(row) => row * 9 + i,
            Self::Column(column) => i * 9 + column,
            Self::Block(block) => (block / 3 * 3 + i / 3) * 9 + block % 3 * 3 + i % 3,
        })
    }
    pub fn label(self) -> String {
        match self {
            Self::Row(row) => format!("第 {} 行", row + 1),
            Self::Column(column) => format!("第 {} 列", column + 1),
            Self::Block(block) => format!("第 {} 宫", block + 1),
        }
    }
}

fn units() -> impl Iterator<Item = Unit> {
    (0..9)
        .map(Unit::Row)
        .chain((0..9).map(Unit::Column))
        .chain((0..9).map(Unit::Block))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FastForwardResult {
    /// Number of digits entered, including a newly entered incorrect digit.
    pub filled: usize,
    /// Candidate marks removed by selected elimination techniques.
    pub eliminated: usize,
    /// The first incorrect entry, including one that existed before the action.
    pub incorrect: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LockedHint {
    pub source: Unit,
    pub target: Unit,
    pub digit: u8,
    pub removals: [bool; 81],
}

/// Per-cell background override used by a hint preview.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HintCellStyle {
    #[default]
    None,
    BlueOutline,
    GreenFill,
    GreenOutline,
    OrangeOutline,
    AlsBlue,
    AlsGreen,
    AlsOrange,
    AlsYellow,
    AlsRed,
}

/// Presentation only: this never changes the board or its candidate masks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HintVisual {
    pub area: [bool; 81],
    pub cells: [HintCellStyle; 81],
    pub highlights: [u16; 81],
    pub removals: [u16; 81],
    pub als_groups: [u8; 81],
}

impl Default for HintVisual {
    fn default() -> Self {
        Self {
            area: [false; 81],
            cells: [HintCellStyle::None; 81],
            highlights: [0; 81],
            removals: [0; 81],
            als_groups: [0; 81],
        }
    }
}

/// A supplied complete solution, checked for Sudoku validity and matching clues.
/// This does not prove the puzzle has a unique solution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Solution([u8; 81]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateIssue {
    Missing { index: usize, digit: u8 },
    Impossible { index: usize, mask: u16 },
}

impl CandidateIssue {
    pub fn index(self) -> usize {
        match self {
            Self::Missing { index, .. } | Self::Impossible { index, .. } => index,
        }
    }
}

impl Solution {
    pub fn parse_for(board: &Board, text: &str) -> Result<Self, &'static str> {
        let complete = Board::parse(text)?;
        if complete.cells.iter().any(|c| c.given == 0)
            || board
                .cells
                .iter()
                .zip(complete.cells.iter())
                .any(|(a, b)| a.given != 0 && a.given != b.given)
        {
            return Err("解答不完整或与题目不符");
        }
        Ok(Self(complete.cells.map(|c| c.given)))
    }
    pub fn digit(&self, index: usize) -> Option<u8> {
        self.0.get(index).copied()
    }
    pub fn first_incorrect_entry(&self, board: &Board) -> Option<usize> {
        board
            .cells
            .iter()
            .enumerate()
            .find_map(|(i, c)| (c.value != 0 && c.value != self.0[i]).then_some(i))
    }
    /// Mirrors the original hint precheck only in automatic mode. Missing notes
    /// are not added until the player explicitly applies the preview.
    pub fn candidate_issue(&self, board: &Board) -> Option<CandidateIssue> {
        if !board.auto_candidates {
            return None;
        }
        for (index, cell) in board.cells.iter().enumerate() {
            if cell.digit() != 0 {
                continue;
            }
            let digit = self.0[index];
            if cell.candidates & (1 << digit) == 0 {
                return Some(CandidateIssue::Missing { index, digit });
            }
            let mask = cell.candidates & !board.legal_candidates(index) & ALL;
            if mask != 0 {
                return Some(CandidateIssue::Impossible { index, mask });
            }
        }
        None
    }
}

impl Board {
    /// Parses exactly 81 ASCII digits; 0 and . denote empty cells.
    pub fn parse(puzzle: &str) -> Result<Self, &'static str> {
        if puzzle.len() != 81 {
            return Err("题目必须包含 81 个字符");
        }
        let mut cells = [Cell::default(); 81];
        for (cell, byte) in cells.iter_mut().zip(puzzle.bytes()) {
            cell.given = match byte {
                b'1'..=b'9' => byte - b'0',
                b'0' | b'.' => 0,
                _ => return Err("题目只能包含数字或点"),
            };
        }
        let mut board = Self {
            cells,
            auto_candidates: false,
        };
        if (0..81).any(|i| board.cells[i].given != 0 && board.conflicts(i)) {
            return Err("题目中存在冲突数字");
        }
        board.reset_candidates();
        board.set_auto_candidates(true);
        Ok(board)
    }
    pub fn cells(&self) -> &[Cell; 81] {
        &self.cells
    }
    pub fn auto_candidates(&self) -> bool {
        self.auto_candidates
    }
    /// Restore editable values and candidate masks without recomputing them.
    /// Validate the entire snapshot before changing any cell.
    pub fn restore_state(
        &mut self,
        values: &[u8],
        candidates: &[u16],
        auto: bool,
    ) -> Result<(), &'static str> {
        if values.len() != 81 || candidates.len() != 81 {
            return Err("存档长度不正确");
        }
        for (i, cell) in self.cells.iter().enumerate() {
            if values[i] > 9
                || candidates[i] & !ALL != 0
                || (cell.given != 0 && values[i] != 0)
                || ((cell.given != 0 || values[i] != 0) && candidates[i] != 0)
            {
                return Err("存档包含无效数据");
            }
        }
        for (i, cell) in self.cells.iter_mut().enumerate() {
            cell.value = values[i];
            cell.candidates = candidates[i];
        }
        self.auto_candidates = auto;
        Ok(())
    }
    pub fn set_auto_candidates(&mut self, enabled: bool) {
        let was_enabled = self.auto_candidates;
        self.auto_candidates = enabled;
        // Only bootstrap a completely empty notes grid when turning auto on.
        // Existing manual notes are never replaced by switching the flag.
        if enabled && !was_enabled && self.cells.iter().all(|c| c.candidates == 0) {
            self.reset_candidates();
        }
    }
    pub fn reset_candidates(&mut self) {
        for i in 0..81 {
            self.cells[i].candidates = if self.cells[i].digit() == 0 {
                self.legal_candidates(i)
            } else {
                0
            };
        }
    }
    pub fn clear_candidates(&mut self) {
        for c in &mut self.cells {
            c.candidates = 0;
        }
    }
    pub fn legal_candidates(&self, index: usize) -> u16 {
        if index >= 81 || self.cells[index].digit() != 0 {
            return 0;
        }
        let mut mask = ALL;
        for j in 0..81 {
            if peers(index, j) {
                mask &= !(1 << self.cells[j].digit());
            }
        }
        mask & ALL
    }
    pub fn set_value(&mut self, index: usize, digit: u8) -> bool {
        if index >= 81
            || digit > 9
            || self.cells[index].given != 0
            || self.cells[index].value == digit
        {
            return false;
        }
        self.cells[index].value = digit;
        self.cells[index].candidates = 0;
        // Automatic mode removes only the newly entered digit from peers.
        // Clearing or replacing a value never recreates previously removed notes.
        if self.auto_candidates && digit != 0 {
            for j in 0..81 {
                if peers(index, j) {
                    self.cells[j].candidates &= !(1 << digit);
                }
            }
        }
        true
    }
    /// Repeatedly fill the first empty cell with exactly one *written note*.
    /// Keep an incorrect entry visible but stop without propagating it to peers.
    /// Correct entries follow the same incremental auto-candidate rule as input.
    pub fn fast_forward(&mut self, solution: &Solution) -> FastForwardResult {
        self.fast_forward_with_techniques(solution, 0)
    }
    /// Written-note singles always run first. Optional, explicitly enabled
    /// techniques are applied only after no written-note single remains.
    pub fn fast_forward_with_techniques(
        &mut self,
        solution: &Solution,
        techniques: u32,
    ) -> FastForwardResult {
        let mut result = FastForwardResult {
            filled: 0,
            eliminated: 0,
            incorrect: solution.first_incorrect_entry(self),
        };
        if result.incorrect.is_some() {
            return result;
        }
        loop {
            let note = self.cells.iter().enumerate().find_map(|(i, cell)| {
                (cell.digit() == 0 && cell.candidates.count_ones() == 1)
                    .then(|| (i, cell.candidates.trailing_zeros() as u8))
            });
            let (index, digit) = if let Some(note) = note {
                note
            } else {
                match self.logical_hint_with_techniques(techniques) {
                    Some(LogicHint::Single(h)) => (h.index, h.digit),
                    Some(LogicHint::Elimination(h)) => {
                        // Do not remove a stored solution digit, even on an ambiguous or invalid position.
                        if (0..81).any(|i| h.removals[i] & (1 << solution.digit(i).unwrap()) != 0) {
                            break;
                        }
                        for i in 0..81 {
                            result.eliminated += h.removals[i].count_ones() as usize;
                            self.cells[i].candidates &= !h.removals[i];
                        }
                        continue;
                    }
                    None => break,
                }
            };
            if solution.digit(index) != Some(digit) {
                let auto = self.auto_candidates;
                self.auto_candidates = false;
                self.set_value(index, digit);
                self.auto_candidates = auto;
                result.filled += 1;
                result.incorrect = Some(index);
                break;
            }
            self.set_value(index, digit);
            result.filled += 1;
        }
        result
    }
    /// Native-style visual stages, using only notes that actually exist.
    pub fn single_hint_visual(&self, hint: Hint, stage: u8) -> HintVisual {
        let mut visual = HintVisual::default();
        visual.cells[hint.index] = HintCellStyle::BlueOutline;
        if stage < 3 {
            return visual;
        }
        if hint.technique == "Naked Single" {
            let row = hint.index / 9;
            let col = hint.index % 9;
            for unit in [
                Unit::Row(row),
                Unit::Column(col),
                Unit::Block(row / 3 * 3 + col / 3),
            ] {
                for i in unit.cells() {
                    visual.area[i] = true;
                }
            }
        } else if let Some(unit) = hint.unit {
            for i in unit.cells() {
                visual.area[i] = true;
            }
        }
        let bit = 1u16 << hint.digit;
        let actual = self.cells[hint.index].candidates;
        visual.highlights[hint.index] = actual & bit;
        visual.removals[hint.index] = actual & !bit;
        if self.auto_candidates {
            for i in 0..81 {
                if peers(hint.index, i) {
                    visual.removals[i] |= self.cells[i].candidates & bit;
                }
            }
        }
        visual
    }

    pub fn locked_hint_visual(&self, hint: LockedHint, stage: u8) -> HintVisual {
        let mut visual = HintVisual::default();
        let source = hint.source.cells();
        let target = hint.target.cells();
        let bit = 1u16 << hint.digit;
        for i in source {
            if target.contains(&i) && self.cells[i].candidates & bit != 0 {
                visual.cells[i] = if stage < 3 {
                    HintCellStyle::BlueOutline
                } else {
                    HintCellStyle::GreenFill
                };
                if stage >= 3 {
                    visual.highlights[i] = bit;
                }
            }
        }
        if stage >= 3 {
            for i in source.into_iter().chain(target) {
                visual.area[i] = true;
            }
            for i in 0..81 {
                if hint.removals[i] && self.cells[i].candidates & bit != 0 {
                    visual.cells[i] = HintCellStyle::OrangeOutline;
                    visual.removals[i] = bit;
                }
            }
        }
        visual
    }

    pub fn candidate_issue_visual(&self, issue: CandidateIssue, stage: u8) -> HintVisual {
        let mut visual = HintVisual::default();
        let index = issue.index();
        visual.cells[index] = if stage < 3 {
            HintCellStyle::BlueOutline
        } else {
            match issue {
                CandidateIssue::Missing { .. } => HintCellStyle::GreenOutline,
                CandidateIssue::Impossible { .. } => HintCellStyle::OrangeOutline,
            }
        };
        if stage >= 3 {
            match issue {
                CandidateIssue::Missing { digit, .. } => {
                    visual.highlights[index] = self.cells[index].candidates & (1 << digit)
                }
                CandidateIssue::Impossible { mask, .. } => {
                    visual.removals[index] = self.cells[index].candidates & mask
                }
            }
        }
        visual
    }

    pub fn incorrect_entry_visual(&self, index: usize, stage: u8) -> HintVisual {
        let mut visual = HintVisual::default();
        visual.cells[index] = if stage < 3 {
            HintCellStyle::BlueOutline
        } else {
            HintCellStyle::OrangeOutline
        };
        visual
    }

    pub fn toggle_candidate(&mut self, index: usize, digit: u8) -> bool {
        if index >= 81 || !(1..=9).contains(&digit) || self.cells[index].digit() != 0 {
            return false;
        }
        self.cells[index].candidates ^= 1 << digit;
        true
    }
    /// Apply only the previewed correction, without recalculating other notes.
    pub fn apply_candidate_issue(&mut self, issue: CandidateIssue) -> bool {
        let index = issue.index();
        if index >= 81 || self.cells[index].digit() != 0 {
            return false;
        }
        let old = self.cells[index].candidates;
        match issue {
            CandidateIssue::Missing { digit, .. } if (1..=9).contains(&digit) => {
                self.cells[index].candidates |= 1 << digit
            }
            CandidateIssue::Impossible { mask, .. } => {
                self.cells[index].candidates &= !(mask & ALL)
            }
            _ => return false,
        }
        old != self.cells[index].candidates
    }
    /// Find a pointing/claiming Locked Candidates elimination. Prove the
    /// confinement from legal digits, but remove only notes actually present.
    /// As in the native dispatcher, this technique requires automatic mode.
    pub fn locked_candidates_hint(&self) -> Option<LockedHint> {
        if !self.auto_candidates
            || (0..81).any(|i| {
                self.conflicts(i) || (self.cells[i].digit() == 0 && self.legal_candidates(i) == 0)
            })
        {
            return None;
        }
        let legal: [u16; 81] = std::array::from_fn(|i| self.legal_candidates(i));
        for block in (0..9).map(Unit::Block) {
            let block_cells = block.cells();
            for line in (0..9).map(Unit::Row).chain((0..9).map(Unit::Column)) {
                let line_cells = line.cells();
                if !block_cells.iter().any(|i| line_cells.contains(i)) {
                    continue;
                }
                for (source, target) in [(block, line), (line, block)] {
                    let source_cells = source.cells();
                    let target_cells = target.cells();
                    for digit in 1..=9 {
                        let bit = 1 << digit;
                        let possible: Vec<_> = source_cells
                            .iter()
                            .copied()
                            .filter(|&i| legal[i] & bit != 0)
                            .collect();
                        if possible.len() < 2 || !possible.iter().all(|i| target_cells.contains(i))
                        {
                            continue;
                        }
                        let mut removals = [false; 81];
                        for i in target_cells {
                            if !source_cells.contains(&i)
                                && legal[i] & self.cells[i].candidates & bit != 0
                            {
                                removals[i] = true;
                            }
                        }
                        if removals.iter().any(|&remove| remove) {
                            return Some(LockedHint {
                                source,
                                target,
                                digit,
                                removals,
                            });
                        }
                    }
                }
            }
        }
        None
    }
    /// Apply a still-current preview as one action. Reject stale or forged previews.
    pub fn apply_locked_candidates(&mut self, hint: LockedHint) -> bool {
        if self.locked_candidates_hint() != Some(hint) {
            return false;
        }
        let bit = 1 << hint.digit;
        for (i, remove) in hint.removals.iter().enumerate() {
            if *remove {
                self.cells[i].candidates &= !bit;
            }
        }
        true
    }
    pub fn conflicts(&self, index: usize) -> bool {
        if index >= 81 {
            return false;
        }
        let digit = self.cells[index].digit();
        digit != 0 && (0..81).any(|j| peers(index, j) && self.cells[j].digit() == digit)
    }
    pub fn is_solved(&self) -> bool {
        (0..81).all(|i| self.cells[i].digit() != 0 && !self.conflicts(i))
    }
    /// Search using legal digits rather than potentially incomplete user notes.
    /// The technique order follows the original dispatcher: Full House,
    /// Naked Single, then Hidden Single. Does not modify the board.
    pub fn hint(&self) -> Option<Hint> {
        if (0..81).any(|i| {
            self.conflicts(i) || (self.cells[i].digit() == 0 && self.legal_candidates(i) == 0)
        }) {
            return None;
        }
        for unit in units() {
            let empty: Vec<_> = unit
                .cells()
                .into_iter()
                .filter(|&i| self.cells[i].digit() == 0)
                .collect();
            if empty.len() == 1 {
                let index = empty[0];
                let mask = self.legal_candidates(index);
                if mask.count_ones() == 1 {
                    return Some(Hint {
                        index,
                        digit: mask.trailing_zeros() as u8,
                        technique: "Full House",
                        unit: Some(unit),
                    });
                }
            }
        }
        for index in 0..81 {
            let mask = self.legal_candidates(index);
            if mask.count_ones() == 1 {
                return Some(Hint {
                    index,
                    digit: mask.trailing_zeros() as u8,
                    technique: "Naked Single",
                    unit: None,
                });
            }
        }
        for unit in units() {
            let cells = unit.cells();
            for digit in 1..=9 {
                if cells.iter().any(|&i| self.cells[i].digit() == digit) {
                    continue;
                }
                let mut matches = cells
                    .into_iter()
                    .filter(|&i| self.legal_candidates(i) & (1 << digit) != 0);
                if let Some(index) = matches.next() {
                    if matches.next().is_none() {
                        return Some(Hint {
                            index,
                            digit,
                            technique: "Hidden Single",
                            unit: Some(unit),
                        });
                    }
                }
            }
        }
        None
    }
}

fn peers(a: usize, b: usize) -> bool {
    a != b && (a / 9 == b / 9 || a % 9 == b % 9 || (a / 27 == b / 27 && a % 9 / 3 == b % 9 / 3))
}

#[cfg(test)]
mod tests {
    use super::*;
    const PUZZLE: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const ANSWER: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
    #[test]
    fn hint_visuals_respect_stages_masks_units_and_manual_mode() {
        let mut board = Board::parse(PUZZLE).unwrap();
        let values = [0; 81];
        let mut notes = [0; 81];
        notes[2] = (1 << 4) | (1 << 2);
        notes[3] = (1 << 4) | (1 << 6);
        board.restore_state(&values, &notes, true).unwrap();
        let before = board.clone();
        let hint = Hint {
            index: 2,
            digit: 4,
            technique: "Naked Single",
            unit: None,
        };
        for stage in [1, 2] {
            let v = board.single_hint_visual(hint, stage);
            assert!(!v.area.iter().any(|x| *x));
            assert_eq!(v.cells[2], HintCellStyle::BlueOutline);
            assert_eq!(v.highlights, [0; 81]);
            assert_eq!(v.removals, [0; 81]);
        }
        let v = board.single_hint_visual(hint, 3);
        assert_eq!(v.area.iter().filter(|x| **x).count(), 21);
        assert_eq!(v.highlights[2], 1 << 4);
        assert_eq!(v.removals[2], 1 << 2);
        assert_eq!(v.removals[3], 1 << 4);
        let full = board.single_hint_visual(
            Hint {
                technique: "Full House",
                unit: Some(Unit::Row(0)),
                ..hint
            },
            3,
        );
        assert_eq!(full.area.iter().filter(|x| **x).count(), 9);
        let hidden = board.single_hint_visual(
            Hint {
                technique: "Hidden Single",
                unit: Some(Unit::Column(2)),
                ..hint
            },
            3,
        );
        assert_eq!(hidden.area.iter().filter(|x| **x).count(), 9);
        assert!(hidden.area[74] && !hidden.area[3]);
        let mut manual = board.clone();
        manual.set_auto_candidates(false);
        assert_eq!(manual.single_hint_visual(hint, 3).removals[3], 0);
        let mut removals = [false; 81];
        removals[3] = true;
        let locked = LockedHint {
            source: Unit::Block(0),
            target: Unit::Row(0),
            digit: 4,
            removals,
        };
        let early = board.locked_hint_visual(locked, 1);
        assert_eq!(early.cells[2], HintCellStyle::BlueOutline);
        assert_eq!(early.cells[3], HintCellStyle::None);
        assert!(!early.area.iter().any(|x| *x));
        let full = board.locked_hint_visual(locked, 3);
        assert_eq!(full.area.iter().filter(|x| **x).count(), 15);
        assert_eq!(full.cells[2], HintCellStyle::GreenFill);
        assert_eq!(full.highlights[2], 1 << 4);
        assert_eq!(full.cells[3], HintCellStyle::OrangeOutline);
        assert_eq!(full.removals[3], 1 << 4);
        let issue = CandidateIssue::Impossible {
            index: 2,
            mask: 1 << 2,
        };
        let v = board.candidate_issue_visual(issue, 3);
        assert_eq!(v.cells[2], HintCellStyle::OrangeOutline);
        assert_eq!(v.removals[2], 1 << 2);
        let missing =
            board.candidate_issue_visual(CandidateIssue::Missing { index: 2, digit: 9 }, 3);
        assert_eq!(missing.cells[2], HintCellStyle::GreenOutline);
        assert_eq!(missing.highlights[2], 0); // absent notes stay invisible
        assert_eq!(board, before);
    }
    #[test]
    fn fast_forward_chains_only_with_auto_and_stops_on_incorrect_note() {
        let make = |auto: bool, first: u8| {
            let mut board = Board::parse(PUZZLE).unwrap();
            let values = [0; 81];
            let mut candidates = [0; 81];
            candidates[2] = 1u16 << first;
            candidates[3] = (1u16 << first) | (1u16 << 6);
            board.restore_state(&values, &candidates, auto).unwrap();
            board
        };
        let mut board = make(true, 4);
        let solution = Solution::parse_for(&board, ANSWER).unwrap();
        assert_eq!(
            board.fast_forward(&solution),
            FastForwardResult {
                filled: 2,
                eliminated: 0,
                incorrect: None
            }
        );
        assert_eq!((board.cells()[2].value, board.cells()[3].value), (4, 6));
        assert_eq!(board.fast_forward(&solution).filled, 0);
        let mut manual = make(false, 4);
        assert_eq!(
            manual.fast_forward(&solution),
            FastForwardResult {
                filled: 1,
                eliminated: 0,
                incorrect: None
            }
        );
        assert_eq!(manual.cells()[3].candidates, (1 << 4) | (1 << 6));
        let mut wrong = make(true, 2);
        assert_eq!(
            wrong.fast_forward(&solution),
            FastForwardResult {
                filled: 1,
                eliminated: 0,
                incorrect: Some(2)
            }
        );
        assert_eq!(wrong.cells()[2].value, 2);
        assert_eq!(wrong.cells()[3].candidates, (1 << 2) | (1 << 6));
        let unchanged = wrong.clone();
        assert_eq!(
            wrong.fast_forward(&solution),
            FastForwardResult {
                filled: 0,
                eliminated: 0,
                incorrect: Some(2)
            }
        );
        assert_eq!(wrong, unchanged);
    }
    #[test]
    fn fast_forward_techniques_are_opt_in_and_filter_individually() {
        let text =
            "034678912072195348198342567859761423426853791713924856961537284287419635345286179";
        let mut initial = Board::parse(text).unwrap();
        // Also remove row 1 column 2, and row 2 column 2: no unit has just one empty.
        initial.cells[1].given = 0;
        initial.cells[10].given = 0;
        initial.clear_candidates();
        let solution = Solution::parse_for(&initial, ANSWER).unwrap();
        let original = initial.clone();
        assert_eq!(
            initial
                .fast_forward_with_techniques(&solution, FULL_HOUSE)
                .filled,
            0
        );
        assert_eq!(initial, original);
        let mut one_left = original.clone();
        one_left.cells[1].given = 3;
        assert!(
            one_left
                .fast_forward_with_techniques(&solution, FULL_HOUSE)
                .filled
                > 0
        );
        let mut naked = original.clone();
        assert_eq!(
            naked
                .fast_forward_with_techniques(&solution, NAKED_SINGLE)
                .filled,
            4
        );
        assert!(naked.is_solved());
        let mut hidden = original.clone();
        assert_eq!(
            hidden
                .fast_forward_with_techniques(&solution, HIDDEN_SINGLE)
                .filled,
            4
        );
        assert!(hidden.is_solved());
        let mut default = original.clone();
        assert_eq!(default.fast_forward(&solution).filled, 0);
        assert_eq!(default, original);
    }
    #[test]
    fn locked_candidates_preview_apply_and_stale_guard() {
        let mut board = Board::parse(PUZZLE).unwrap();
        let before = board.clone();
        let hint = board.locked_candidates_hint().unwrap();
        assert_eq!(
            (hint.source, hint.target, hint.digit),
            (Unit::Block(0), Unit::Row(1), 7)
        );
        assert_eq!(
            hint.removals
                .iter()
                .enumerate()
                .filter_map(|(i, &v)| v.then_some(i))
                .collect::<Vec<_>>(),
            vec![15, 17]
        );
        assert_eq!(board, before);
        assert!(board.apply_locked_candidates(hint));
        for i in 0..81 {
            let expected = if hint.removals[i] {
                before.cells()[i].candidates & !(1 << 7)
            } else {
                before.cells()[i].candidates
            };
            assert_eq!(board.cells()[i].candidates, expected);
        }
        assert!(!board.apply_locked_candidates(hint));
        board = before.clone();
        board.set_auto_candidates(false);
        assert!(board.locked_candidates_hint().is_none());
        assert!(!board.apply_locked_candidates(hint));
    }
    #[test]
    fn locked_candidates_cover_pointing_and_claiming_without_removing_solution() {
        let mut board = Board::parse(PUZZLE).unwrap();
        let solution = Solution::parse_for(&board, ANSWER).unwrap();
        let (mut pointing, mut claiming) = (false, false);
        for _ in 0..500 {
            let Some(hint) = board.locked_candidates_hint() else {
                break;
            };
            pointing |= matches!(hint.source, Unit::Block(_));
            claiming |= !matches!(hint.source, Unit::Block(_));
            for i in 0..81 {
                if hint.removals[i] {
                    assert_ne!(solution.digit(i), Some(hint.digit));
                }
            }
            assert!(board.apply_locked_candidates(hint));
        }
        assert!(pointing && claiming);
    }
    #[test]
    fn full_house_precedes_naked_single_and_is_a_pure_preview() {
        let mut text = ANSWER.as_bytes().to_vec();
        text[2] = b'0';
        let mut board = Board::parse(std::str::from_utf8(&text).unwrap()).unwrap();
        board.clear_candidates();
        let before = board.clone();
        assert_eq!(
            board.hint(),
            Some(Hint {
                index: 2,
                digit: 4,
                technique: "Full House",
                unit: Some(Unit::Row(0))
            })
        );
        assert_eq!(board, before);
    }
    #[test]
    fn hidden_singles_cover_rows_columns_and_blocks() {
        let fixtures = [
            (
                "030008000000000308100000060000000000020050090010020056000500204080409005000080070",
                20,
                8,
                Unit::Row(2),
            ),
            (
                "500608900072100040008340000809000003006803091700900000000000000080400030000200000",
                15,
                3,
                Unit::Column(6),
            ),
            (
                "534078900602000000008040000800000003000000000010000006000007204000000600040200000",
                18,
                1,
                Unit::Block(0),
            ),
        ];
        for (puzzle, index, digit, unit) in fixtures {
            let board = Board::parse(puzzle).unwrap();
            assert!(board.legal_candidates(index).count_ones() > 1);
            let before = board.clone();
            assert_eq!(
                board.hint(),
                Some(Hint {
                    index,
                    digit,
                    technique: "Hidden Single",
                    unit: Some(unit)
                })
            );
            assert_eq!(before, board);
            assert_eq!(
                Solution::parse_for(&board, ANSWER).unwrap().digit(index),
                Some(digit)
            );
        }
    }
    #[test]
    fn conflicting_or_impossible_board_has_no_hint() {
        let mut board = Board::parse(PUZZLE).unwrap();
        board.set_value(2, 5);
        assert!(board.hint().is_none());
    }
    #[test]
    fn supplied_solution_and_candidate_previews() {
        let mut b = Board::parse(PUZZLE).unwrap();
        let solution = Solution::parse_for(&b, ANSWER).unwrap();
        assert!(Solution::parse_for(&b, &"1".repeat(81)).is_err());
        assert!(Solution::parse_for(&b, &"0".repeat(81)).is_err());
        assert_eq!(solution.candidate_issue(&b), None);
        b.toggle_candidate(2, 4);
        let missing = CandidateIssue::Missing { index: 2, digit: 4 };
        assert_eq!(solution.candidate_issue(&b), Some(missing));
        let snapshot = b.clone();
        assert_eq!(b, snapshot); // preview is pure
        assert!(b.apply_candidate_issue(missing));
        assert_ne!(b.cells()[2].candidates & (1 << 4), 0);
        assert_eq!(solution.candidate_issue(&b), None);
        b.toggle_candidate(2, 9);
        let impossible = CandidateIssue::Impossible {
            index: 2,
            mask: 1 << 9,
        };
        assert_eq!(solution.candidate_issue(&b), Some(impossible));
        assert!(b.apply_candidate_issue(impossible));
        b.set_auto_candidates(false);
        b.toggle_candidate(2, 4);
        assert_eq!(solution.candidate_issue(&b), None);
        b.set_auto_candidates(true);
        assert_eq!(solution.candidate_issue(&b), Some(missing));
        b.set_value(2, 1);
        assert_eq!(solution.first_incorrect_entry(&b), Some(2));
    }
    #[test]
    fn hint_does_not_treat_partial_notes_as_a_naked_single() {
        let mut b = Board::parse(PUZZLE).unwrap();
        b.clear_candidates();
        b.toggle_candidate(2, 1);
        assert_ne!(b.hint().map(|h| h.index), Some(2));
    }
    #[test]
    fn parses_and_rejects_invalid() {
        assert!(Board::parse(PUZZLE).is_ok());
        assert!(Board::parse("short").is_err());
        assert!(Board::parse(&"1".repeat(81)).is_err());
        assert!(Board::parse(&"é".repeat(40)).is_err());
    }
    #[test]
    fn candidates_follow_peers_and_include_nine() {
        let mut b = Board::parse(PUZZLE).unwrap();
        assert_eq!(b.cells()[2].candidates, (1 << 1) | (1 << 2) | (1 << 4));
        assert_ne!(
            Board::parse(&"0".repeat(81)).unwrap().cells()[0].candidates & (1 << 9),
            0
        );
        assert!(b.set_value(2, 1));
        assert_eq!(b.cells()[6].candidates & (1 << 1), 0);
        assert!(b.set_value(2, 0));
        assert_eq!(b.cells()[6].candidates & (1 << 1), 0);
    }
    #[test]
    fn auto_toggle_and_incremental_notes() {
        let mut b = Board::parse(&"0".repeat(81)).unwrap();
        assert!(b.toggle_candidate(1, 8));
        let notes = b.cells().map(|c| c.candidates);
        b.set_auto_candidates(false);
        b.set_auto_candidates(true);
        assert_eq!(notes, b.cells().map(|c| c.candidates));
        assert!(b.set_value(0, 1));
        assert_eq!(b.cells()[1].candidates, notes[1] & !(1 << 1));
        assert_eq!(b.cells()[40].candidates, notes[40]);
        assert!(b.set_value(0, 2));
        assert_eq!(b.cells()[1].candidates, notes[1] & !(1 << 1) & !(1 << 2));
        assert!(b.set_value(0, 0));
        assert_eq!(b.cells()[1].candidates, notes[1] & !(1 << 1) & !(1 << 2));
        b.set_auto_candidates(false);
        let before = b.cells()[1].candidates;
        assert!(b.set_value(0, 3));
        assert_eq!(b.cells()[1].candidates, before);
        b.reset_candidates();
        assert_eq!(b.cells()[1].candidates, b.legal_candidates(1));
        b.clear_candidates();
        assert!(b.cells().iter().all(|c| c.candidates == 0));
    }
    #[test]
    fn enabling_auto_bootstraps_only_an_empty_notes_grid() {
        let mut b = Board::parse(PUZZLE).unwrap();
        b.set_auto_candidates(false);
        b.clear_candidates();
        b.set_auto_candidates(true);
        assert_eq!(b.cells()[2].candidates, b.legal_candidates(2));
        assert_ne!(b.cells()[2].candidates, 0);
        b.set_auto_candidates(false);
        b.clear_candidates();
        assert!(b.toggle_candidate(2, 9));
        let notes = b.cells().map(|c| c.candidates);
        b.set_auto_candidates(true);
        assert_eq!(b.cells().map(|c| c.candidates), notes);
        b.clear_candidates();
        b.set_auto_candidates(true); // already on: no implicit reset
        assert!(b.cells().iter().all(|c| c.candidates == 0));
    }
    #[test]
    fn restore_is_validated_and_atomic() {
        let mut b = Board::parse(PUZZLE).unwrap();
        let initial = b.clone();
        let mut values = vec![0; 81];
        let mut candidates = b.cells().map(|c| c.candidates).to_vec();
        values[2] = 4;
        candidates[2] = 0;
        assert!(b.restore_state(&values, &candidates, false).is_ok());
        assert_eq!(b.cells()[2].value, 4);
        assert!(!b.auto_candidates());
        let saved = b.clone();
        values[0] = 1;
        assert!(b.restore_state(&values, &candidates, true).is_err());
        assert_eq!(b, saved);
        values[0] = 0;
        candidates[2] = 0x8000;
        assert!(b.restore_state(&values, &candidates, true).is_err());
        assert_eq!(b, saved);
        assert!(b.restore_state(&values[..80], &candidates, true).is_err());
        assert_eq!(b, saved);
        assert_ne!(b, initial);
    }
    #[test]
    fn editing_and_conflicts() {
        let mut b = Board::parse(PUZZLE).unwrap();
        assert!(!b.set_value(0, 2));
        assert!(!b.set_value(100, 1));
        assert!(b.set_value(2, 5));
        assert!(b.conflicts(0));
        assert!(b.conflicts(2));
        assert!(!b.is_solved());
    }
    #[test]
    fn manual_candidates_and_preview() {
        let mut b = Board::parse(&"0".repeat(81)).unwrap();
        b.set_auto_candidates(false);
        b.clear_candidates();
        assert!(b.toggle_candidate(0, 9));
        let before = b.clone();
        assert_eq!(b.hint(), None);
        assert_eq!(before, b);
        let demo = Board::parse(PUZZLE).unwrap();
        let preview = demo.hint().unwrap();
        assert_eq!(demo.legal_candidates(preview.index), 1 << preview.digit);
        assert!(!b.toggle_candidate(0, 0));
    }
}
