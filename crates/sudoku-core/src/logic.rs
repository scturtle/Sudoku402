//! Reconstruct deductions from legal candidates, never trusting missing player notes as proof.
use super::*;
pub const FULL_HOUSE: u32 = 1 << 0;
pub const NAKED_SINGLE: u32 = 1 << 1;
pub const HIDDEN_SINGLE: u32 = 1 << 2;
pub const HIDDEN_PAIR: u32 = 1 << 3;
pub const NAKED_PAIR: u32 = 1 << 4;
pub const LOCKED_CANDIDATES: u32 = 1 << 5;
pub const HIDDEN_TRIPLE: u32 = 1 << 6;
pub const HIDDEN_QUAD: u32 = 1 << 7;
pub const X_WING: u32 = 1 << 8;
pub const NAKED_TRIPLE: u32 = 1 << 9;
pub const NAKED_QUAD: u32 = 1 << 10;
pub const SWORDFISH: u32 = 1 << 11;
pub const JELLYFISH: u32 = 1 << 12;
pub const FINNED_X_WING: u32 = 1 << 13;
pub const XY_WING: u32 = 1 << 14;
pub const SQUIRMBAG: u32 = 1 << 15;
pub const FINNED_SWORDFISH: u32 = 1 << 16;
pub const FINNED_JELLYFISH: u32 = 1 << 17;
pub const XYZ_WING: u32 = 1 << 18;
pub const WXYZ_WING: u32 = 1 << 19;
pub const ALMOST_LOCKED_SETS: u32 = 1 << 20;
pub const FINNED_SQUIRMBAG: u32 = 1 << 21;
pub const ALS_CHAIN: u32 = 1 << 22;
pub const ALL_TECHNIQUES: u32 = (1 << 23) - 1;
fn technique_bit(name: &str) -> u32 {
    match name {
        "Full House" => FULL_HOUSE,
        "Naked Single" => NAKED_SINGLE,
        "Hidden Single" => HIDDEN_SINGLE,
        "Hidden Pair" => HIDDEN_PAIR,
        "Naked Pair" => NAKED_PAIR,
        "Locked Candidates" => LOCKED_CANDIDATES,
        "Hidden Triple" => HIDDEN_TRIPLE,
        "Hidden Quad" => HIDDEN_QUAD,
        "X-Wing" => X_WING,
        "Naked Triple" => NAKED_TRIPLE,
        "Naked Quad" => NAKED_QUAD,
        "Swordfish" => SWORDFISH,
        "Jellyfish" => JELLYFISH,
        "Finned X-Wing" => FINNED_X_WING,
        "XY-Wing" => XY_WING,
        "Squirmbag" => SQUIRMBAG,
        "Finned Swordfish" => FINNED_SWORDFISH,
        "Finned Jellyfish" => FINNED_JELLYFISH,
        "XYZ-Wing" => XYZ_WING,
        "WXYZ-Wing" => WXYZ_WING,
        "Almost Locked Sets" => ALMOST_LOCKED_SETS,
        "Finned Squirmbag" => FINNED_SQUIRMBAG,
        "ALS-Chain" => ALS_CHAIN,
        _ => 0,
    }
}

fn block_index(i: usize) -> usize {
    (i / 27) * 3 + (i % 9) / 3
}
fn sees(a: usize, b: usize) -> bool {
    a != b && (a / 9 == b / 9 || a % 9 == b % 9 || block_index(a) == block_index(b))
}
// Is there a peer-consistent assignment to this small wing without digit z?
fn wing_without_z(
    cells: &[usize; 4],
    masks: &[u16; 81],
    z: u16,
    pos: usize,
    chosen: &mut [u16; 4],
) -> bool {
    if pos == 4 {
        return true;
    }
    let mut bits = masks[cells[pos]] & !z;
    while bits != 0 {
        let bit = bits & bits.wrapping_neg();
        bits &= !bit;
        if (0..pos).any(|j| chosen[j] == bit && sees(cells[j], cells[pos])) {
            continue;
        }
        chosen[pos] = bit;
        if wing_without_z(cells, masks, z, pos + 1, chosen) {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EliminationHint {
    pub technique: &'static str,
    pub unit: Unit,
    pub second_unit: Option<Unit>,
    pub fins: [bool; 81],
    pub cover_lines: u16,
    /// 1-based ordered ALS groups and their restricted common digits.
    pub als_groups: [u8; 81],
    pub als_links: [u16; 5],
    pub structure: [u16; 81],
    pub removals: [u16; 81],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicHint {
    Single(Hint),
    Elimination(EliminationHint),
}

impl Board {
    /// Reconstruct only deductions whose removals are already absent from notes.
    /// Every inference starts with legal candidates, so arbitrary missing notes
    /// cannot by themselves establish a false pair, confinement or single.
    pub fn logical_hint(&self) -> Option<LogicHint> {
        self.logical_hint_with_techniques(ALL_TECHNIQUES)
    }
    /// Only enabled techniques create new actions. Earlier independently proven
    /// eliminations can be reconstructed, even if that technique is now disabled.
    pub fn logical_hint_with_techniques(&self, techniques: u32) -> Option<LogicHint> {
        if techniques & ALL_TECHNIQUES == 0 {
            return None;
        }
        if (0..81).any(|i| {
            self.conflicts(i) || (self.cells[i].digit() == 0 && self.legal_candidates(i) == 0)
        }) {
            return None;
        }
        let mut masks: [u16; 81] = std::array::from_fn(|i| self.legal_candidates(i));
        loop {
            if let Some(h) = self.single_from_masks(&masks, techniques) {
                return Some(LogicHint::Single(h));
            }
            if !self.auto_candidates {
                return None;
            }
            let mut found = None;
            self.visit_eliminations(&masks, |h| {
                let already_absent = (0..81).all(|i| h.removals[i] & self.cells[i].candidates == 0);
                if !already_absent && techniques & technique_bit(h.technique) == 0 {
                    return false;
                }
                found = Some(h);
                true
            });
            let h = found?;
            let mut preview = h;
            for i in 0..81 {
                preview.removals[i] &= self.cells[i].candidates;
            }
            if preview.removals.iter().any(|&m| m != 0) {
                return Some(LogicHint::Elimination(preview));
            }
            // Reconstruct a proven deduction only when its targets are already absent.
            let mut changed = false;
            for i in 0..81 {
                let next = masks[i] & !h.removals[i];
                if masks[i] != next {
                    changed = true;
                    masks[i] = next;
                }
                if self.cells[i].digit() == 0 && masks[i] == 0 {
                    return None;
                }
            }
            if !changed {
                return None;
            }
        }
    }
    fn single_from_masks(&self, masks: &[u16; 81], techniques: u32) -> Option<Hint> {
        if techniques & FULL_HOUSE != 0 {
            for unit in units() {
                let empty: Vec<_> = unit
                    .cells()
                    .into_iter()
                    .filter(|&i| self.cells[i].digit() == 0)
                    .collect();
                if empty.len() == 1 && masks[empty[0]].count_ones() == 1 {
                    return Some(Hint {
                        index: empty[0],
                        digit: masks[empty[0]].trailing_zeros() as u8,
                        technique: "Full House",
                        unit: Some(unit),
                    });
                }
            }
        }
        if techniques & NAKED_SINGLE != 0 {
            for i in 0..81 {
                if masks[i].count_ones() == 1 {
                    return Some(Hint {
                        index: i,
                        digit: masks[i].trailing_zeros() as u8,
                        technique: "Naked Single",
                        unit: None,
                    });
                }
            }
        }
        if techniques & HIDDEN_SINGLE == 0 {
            return None;
        }
        for unit in units() {
            let cells = unit.cells();
            for digit in 1..=9 {
                if cells.iter().any(|&i| self.cells[i].digit() == digit) {
                    continue;
                }
                let mut found = cells.into_iter().filter(|&i| masks[i] & (1 << digit) != 0);
                if let Some(index) = found.next() {
                    if found.next().is_none() {
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
    fn visit_eliminations(
        &self,
        masks: &[u16; 81],
        mut visit: impl FnMut(EliminationHint) -> bool,
    ) {
        // Web search order; native dispatch statically constructs Naked Pair first.
        for technique in ["Hidden Pair", "Naked Pair"] {
            for unit in units() {
                let cells = unit.cells();
                for a in 1..=8 {
                    for b in a + 1..=9 {
                        let bits = (1 << a) | (1 << b);
                        let mut h = EliminationHint {
                            technique,
                            unit,
                            second_unit: None,
                            fins: [false; 81],
                            als_groups: [0; 81],
                            als_links: [0; 5],
                            cover_lines: 0,
                            structure: [0; 81],
                            removals: [0; 81],
                        };
                        if technique == "Hidden Pair" {
                            let aa: Vec<_> = cells
                                .iter()
                                .copied()
                                .filter(|&i| masks[i] & (1 << a) != 0)
                                .collect();
                            let bb: Vec<_> = cells
                                .iter()
                                .copied()
                                .filter(|&i| masks[i] & (1 << b) != 0)
                                .collect();
                            if aa.len() != 2 || aa != bb {
                                continue;
                            }
                            for i in aa {
                                h.structure[i] = masks[i] & bits;
                                h.removals[i] = masks[i] & !bits;
                            }
                        } else {
                            let pair: Vec<_> = cells
                                .iter()
                                .copied()
                                .filter(|&i| masks[i] == bits)
                                .collect();
                            if pair.len() != 2 {
                                continue;
                            }
                            for &i in &pair {
                                h.structure[i] = bits;
                            }
                            for i in cells {
                                if !pair.contains(&i) {
                                    h.removals[i] = masks[i] & bits;
                                }
                            }
                        }
                        if h.removals.iter().any(|&m| m != 0) && visit(h) {
                            return;
                        }
                    }
                }
            }
        }
        for block in (0..9).map(Unit::Block) {
            let bc = block.cells();
            for line in (0..9).map(Unit::Row).chain((0..9).map(Unit::Column)) {
                let lc = line.cells();
                if !bc.iter().any(|i| lc.contains(i)) {
                    continue;
                }
                for (source, target) in [(block, line), (line, block)] {
                    let sc = source.cells();
                    let tc = target.cells();
                    for digit in 1..=9 {
                        let bit = 1 << digit;
                        let hits: Vec<_> = sc
                            .iter()
                            .copied()
                            .filter(|&i| masks[i] & bit != 0)
                            .collect();
                        if hits.len() < 2 || !hits.iter().all(|i| tc.contains(i)) {
                            continue;
                        }
                        let mut h = EliminationHint {
                            technique: "Locked Candidates",
                            unit: source,
                            second_unit: Some(target),
                            fins: [false; 81],
                            als_groups: [0; 81],
                            als_links: [0; 5],
                            cover_lines: 0,
                            structure: [0; 81],
                            removals: [0; 81],
                        };
                        for i in hits {
                            h.structure[i] = bit;
                        }
                        for i in tc {
                            if !sc.contains(&i) {
                                h.removals[i] = masks[i] & bit;
                            }
                        }
                        if h.removals.iter().any(|&m| m != 0) && visit(h) {
                            return;
                        }
                    }
                }
            }
        }
        // Web order, not native action dispatch: hidden triples/quads, fish, then naked triples/quads.
        for technique in [
            "Hidden Triple",
            "Hidden Quad",
            "X-Wing",
            "Naked Triple",
            "Naked Quad",
        ] {
            if technique == "X-Wing" {
                for digit in 1..=9 {
                    let bit = 1 << digit;
                    for rows in [true, false] {
                        for a in 0..8 {
                            for b in a + 1..9 {
                                let unit_a = if rows { Unit::Row(a) } else { Unit::Column(a) };
                                let unit_b = if rows { Unit::Row(b) } else { Unit::Column(b) };
                                let cells_a = unit_a.cells();
                                let cells_b = unit_b.cells();
                                let positions_a = (0..9)
                                    .filter(|&p| masks[cells_a[p]] & bit != 0)
                                    .collect::<Vec<_>>();
                                let positions_b = (0..9)
                                    .filter(|&p| masks[cells_b[p]] & bit != 0)
                                    .collect::<Vec<_>>();
                                if positions_a.len() != 2 || positions_a != positions_b {
                                    continue;
                                }
                                let mut h = EliminationHint {
                                    technique,
                                    unit: unit_a,
                                    second_unit: Some(unit_b),
                                    fins: [false; 81],
                                    als_groups: [0; 81],
                                    als_links: [0; 5],
                                    cover_lines: 0,
                                    structure: [0; 81],
                                    removals: [0; 81],
                                };
                                for &p in &positions_a {
                                    h.structure[cells_a[p]] = bit;
                                    h.structure[cells_b[p]] = bit;
                                }
                                for line in 0..9 {
                                    if line == a || line == b {
                                        continue;
                                    }
                                    for &p in &positions_a {
                                        let i = if rows { line * 9 + p } else { p * 9 + line };
                                        h.removals[i] = masks[i] & bit;
                                    }
                                }
                                if h.removals.iter().any(|&m| m != 0) && visit(h) {
                                    return;
                                }
                            }
                        }
                    }
                }
                continue;
            }
            let hidden = technique.starts_with("Hidden");
            let size = if technique.ends_with("Triple") { 3 } else { 4 };
            for unit in units() {
                let cells = unit.cells();
                for bits in (2u16..=0x3fe).filter(|m| m & 1 == 0 && m.count_ones() == size) {
                    let mut h = EliminationHint {
                        technique,
                        unit,
                        second_unit: None,
                        fins: [false; 81],
                        als_groups: [0; 81],
                        als_links: [0; 5],
                        cover_lines: 0,
                        structure: [0; 81],
                        removals: [0; 81],
                    };
                    if hidden {
                        let selected: Vec<_> = cells
                            .iter()
                            .copied()
                            .filter(|&i| masks[i] & bits != 0)
                            .collect();
                        if selected.len() != size as usize
                            || selected.iter().fold(0, |all, &i| all | (masks[i] & bits)) != bits
                        {
                            continue;
                        }
                        for i in selected {
                            h.structure[i] = masks[i] & bits;
                            h.removals[i] = masks[i] & !bits;
                        }
                    } else {
                        let selected: Vec<_> = cells
                            .iter()
                            .copied()
                            .filter(|&i| masks[i] != 0 && masks[i] & !bits == 0)
                            .collect();
                        if selected.len() != size as usize
                            || selected.iter().fold(0, |all, &i| all | masks[i]) != bits
                        {
                            continue;
                        }
                        for &i in &selected {
                            h.structure[i] = masks[i];
                        }
                        for i in cells {
                            if !selected.contains(&i) {
                                h.removals[i] = masks[i] & bits;
                            }
                        }
                    }
                    if h.removals.iter().any(|&m| m != 0) && visit(h) {
                        return;
                    }
                }
            }
        }
        self.visit_hard_eliminations(masks, visit);
    }
    fn visit_hard_eliminations(
        &self,
        masks: &[u16; 81],
        mut visit: impl FnMut(EliminationHint) -> bool,
    ) {
        // Ordinary fish: N base lines use exactly N cover lines.
        for (technique, n) in [("Swordfish", 3u32), ("Jellyfish", 4), ("Squirmbag", 5)] {
            for rows in [true, false] {
                for digit in 1..=9 {
                    let bit = 1u16 << digit;
                    let positions: [u16; 9] = std::array::from_fn(|line| {
                        (0..9).fold(0, |acc, p| {
                            let i = if rows { line * 9 + p } else { p * 9 + line };
                            acc | if masks[i] & bit != 0 { 1 << p } else { 0 }
                        })
                    });
                    for bases in (1u16..512).filter(|m| m.count_ones() == n) {
                        let lines: Vec<_> = (0..9).filter(|&i| bases & (1 << i) != 0).collect();
                        if lines.iter().any(|&i| {
                            positions[i].count_ones() < 2 || positions[i].count_ones() > n
                        }) {
                            continue;
                        }
                        let cover = lines.iter().fold(0, |m, &i| m | positions[i]);
                        if cover.count_ones() != n {
                            continue;
                        }
                        let unit = |i| if rows { Unit::Row(i) } else { Unit::Column(i) };
                        let mut h = EliminationHint {
                            technique,
                            unit: unit(lines[0]),
                            second_unit: Some(unit(lines[1])),
                            fins: [false; 81],
                            als_groups: [0; 81],
                            als_links: [0; 5],
                            cover_lines: cover,
                            structure: [0; 81],
                            removals: [0; 81],
                        };
                        for line in 0..9 {
                            for p in 0..9 {
                                let i = if rows { line * 9 + p } else { p * 9 + line };
                                if bases & (1 << line) != 0 {
                                    h.structure[i] = masks[i] & bit;
                                } else if cover & (1 << p) != 0 {
                                    h.removals[i] = masks[i] & bit;
                                }
                            }
                        }
                        if h.removals.iter().any(|&m| m != 0) && visit(h) {
                            return;
                        }
                    }
                }
            }
        }
        // A finned X-Wing: all off-cover candidates lie in one block.
        // A target sees every fin and lies on a cover outside the base lines.
        for rows in [true, false] {
            for digit in 1..=9 {
                let bit = 1u16 << digit;
                let positions: [u16; 9] = std::array::from_fn(|line| {
                    (0..9).fold(0, |acc, p| {
                        let i = if rows { line * 9 + p } else { p * 9 + line };
                        acc | if masks[i] & bit != 0 { 1 << p } else { 0 }
                    })
                });
                for a in 0..8 {
                    for b in a + 1..9 {
                        for cover in (1u16..512).filter(|m| m.count_ones() == 2) {
                            if positions[a] & cover == 0 || positions[b] & cover == 0 {
                                continue;
                            }
                            let extra_a = positions[a] & !cover;
                            let extra_b = positions[b] & !cover;
                            if (extra_a == 0) == (extra_b == 0) {
                                continue;
                            }
                            let fin_line = if extra_a != 0 { a } else { b };
                            let extra = extra_a | extra_b;
                            let fins: Vec<_> = (0..9)
                                .filter(|&p| extra & (1 << p) != 0)
                                .map(|p| {
                                    if rows {
                                        fin_line * 9 + p
                                    } else {
                                        p * 9 + fin_line
                                    }
                                })
                                .collect();
                            if fins.iter().any(|&i| block_index(i) != block_index(fins[0])) {
                                continue;
                            }
                            let unit = |i| if rows { Unit::Row(i) } else { Unit::Column(i) };
                            let mut h = EliminationHint {
                                technique: "Finned X-Wing",
                                unit: unit(a),
                                second_unit: Some(unit(b)),
                                fins: [false; 81],
                                als_groups: [0; 81],
                                als_links: [0; 5],
                                cover_lines: cover,
                                structure: [0; 81],
                                removals: [0; 81],
                            };
                            for &i in &fins {
                                h.fins[i] = true;
                            }
                            for line in [a, b] {
                                for p in 0..9 {
                                    let i = if rows { line * 9 + p } else { p * 9 + line };
                                    h.structure[i] = masks[i] & bit;
                                }
                            }
                            for line in 0..9 {
                                if line == a || line == b {
                                    continue;
                                }
                                for p in 0..9 {
                                    if cover & (1 << p) == 0 {
                                        continue;
                                    }
                                    let i = if rows { line * 9 + p } else { p * 9 + line };
                                    if fins.iter().all(|&f| sees(i, f)) {
                                        h.removals[i] = masks[i] & bit;
                                    }
                                }
                            }
                            if h.removals.iter().any(|&m| m != 0) && visit(h) {
                                return;
                            }
                        }
                    }
                }
            }
        }
        // XY-Wing: pivot XY sees wings XZ and YZ; each target sees both wings.
        let bivalue: Vec<_> = (0..81).filter(|&i| masks[i].count_ones() == 2).collect();
        for &pivot in &bivalue {
            for (pos, &a) in bivalue.iter().enumerate() {
                if !sees(pivot, a) {
                    continue;
                }
                let pa = masks[pivot] & masks[a];
                if pa.count_ones() != 1 {
                    continue;
                }
                for &b in &bivalue[pos + 1..] {
                    if !sees(pivot, b) {
                        continue;
                    }
                    let pb = masks[pivot] & masks[b];
                    let z = masks[a] & masks[b];
                    if pb.count_ones() != 1
                        || pb == pa
                        || z.count_ones() != 1
                        || z & masks[pivot] != 0
                    {
                        continue;
                    }
                    let mut h = EliminationHint {
                        technique: "XY-Wing",
                        unit: Unit::Row(pivot / 9),
                        second_unit: None,
                        fins: [false; 81],
                        als_groups: [0; 81],
                        als_links: [0; 5],
                        cover_lines: 0,
                        structure: [0; 81],
                        removals: [0; 81],
                    };
                    h.structure[pivot] = masks[pivot];
                    h.structure[a] = masks[a];
                    h.structure[b] = masks[b];
                    for i in 0..81 {
                        if i != pivot && i != a && i != b && sees(i, a) && sees(i, b) {
                            h.removals[i] = masks[i] & z;
                        }
                    }
                    if h.removals.iter().any(|&m| m != 0) && visit(h) {
                        return;
                    }
                }
            }
        }
        // Larger finned fish: choose N-1 clean lines, then one finned line.
        // If no fin is true, the N base lines occupy N covers; otherwise every
        // target seeing all fins is excluded. The same proof works in either orientation.
        for (technique, n) in [
            ("Finned Swordfish", 3u32),
            ("Finned Jellyfish", 4),
            ("Finned Squirmbag", 5),
        ] {
            for rows in [true, false] {
                for digit in 1..=9 {
                    let bit = 1u16 << digit;
                    let positions: [u16; 9] = std::array::from_fn(|line| {
                        (0..9).fold(0, |acc, p| {
                            let i = if rows { line * 9 + p } else { p * 9 + line };
                            acc | if masks[i] & bit != 0 { 1 << p } else { 0 }
                        })
                    });
                    for clean in (1u16..512).filter(|m| m.count_ones() == n - 1) {
                        let lines: Vec<_> = (0..9).filter(|&i| clean & (1 << i) != 0).collect();
                        if lines.iter().any(|&i| {
                            positions[i].count_ones() < 2 || positions[i].count_ones() > n
                        }) {
                            continue;
                        }
                        let union = lines.iter().fold(0u16, |a, &i| a | positions[i]);
                        let covers: Vec<u16> = if union.count_ones() == n {
                            vec![union]
                        } else if union.count_ones() == n - 1 {
                            (0..9)
                                .filter(|&p| union & (1 << p) == 0)
                                .map(|p| union | (1 << p))
                                .collect()
                        } else {
                            continue;
                        };
                        for cover in covers {
                            for finline in 0..9 {
                                if clean & (1 << finline) != 0 || positions[finline] & cover == 0 {
                                    continue;
                                }
                                let extra = positions[finline] & !cover;
                                if extra == 0 {
                                    continue;
                                }
                                let fins: Vec<_> = (0..9)
                                    .filter(|&p| extra & (1 << p) != 0)
                                    .map(|p| {
                                        if rows {
                                            finline * 9 + p
                                        } else {
                                            p * 9 + finline
                                        }
                                    })
                                    .collect();
                                if fins.iter().any(|&i| block_index(i) != block_index(fins[0])) {
                                    continue;
                                }
                                let unit = |i| if rows { Unit::Row(i) } else { Unit::Column(i) };
                                let mut h = EliminationHint {
                                    technique,
                                    unit: unit(lines[0]),
                                    second_unit: Some(unit(finline)),
                                    fins: [false; 81],
                                    als_groups: [0; 81],
                                    als_links: [0; 5],
                                    cover_lines: cover,
                                    structure: [0; 81],
                                    removals: [0; 81],
                                };
                                for &i in &fins {
                                    h.fins[i] = true;
                                }
                                for line in 0..9 {
                                    for p in 0..9 {
                                        let i = if rows { line * 9 + p } else { p * 9 + line };
                                        if line == finline || clean & (1 << line) != 0 {
                                            h.structure[i] = masks[i] & bit;
                                        } else if cover & (1 << p) != 0
                                            && fins.iter().all(|&f| sees(i, f))
                                        {
                                            h.removals[i] = masks[i] & bit;
                                        }
                                    }
                                }
                                if h.removals.iter().any(|&m| m != 0) && visit(h) {
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }
        // XYZ-Wing: pivot XYZ and two bivalue wings XZ and YZ.
        for pivot in 0..81 {
            if masks[pivot].count_ones() != 3 {
                continue;
            }
            for (pos, &a) in bivalue.iter().enumerate() {
                if !sees(pivot, a) || masks[a] & !masks[pivot] != 0 {
                    continue;
                }
                for &b in &bivalue[pos + 1..] {
                    if !sees(pivot, b) || masks[b] & !masks[pivot] != 0 || masks[a] == masks[b] {
                        continue;
                    }
                    let z = masks[a] & masks[b];
                    if z.count_ones() != 1 || masks[a] | masks[b] != masks[pivot] {
                        continue;
                    }
                    let mut h = EliminationHint {
                        technique: "XYZ-Wing",
                        unit: Unit::Row(pivot / 9),
                        second_unit: None,
                        fins: [false; 81],
                        als_groups: [0; 81],
                        als_links: [0; 5],
                        cover_lines: 0,
                        structure: [0; 81],
                        removals: [0; 81],
                    };
                    for i in [pivot, a, b] {
                        h.structure[i] = masks[i];
                    }
                    for i in 0..81 {
                        if i != pivot
                            && i != a
                            && i != b
                            && sees(i, pivot)
                            && sees(i, a)
                            && sees(i, b)
                        {
                            h.removals[i] = masks[i] & z;
                        }
                    }
                    if h.removals.iter().any(|&m| m != 0) && visit(h) {
                        return;
                    }
                }
            }
        }
        // Two disjoint almost-locked sets with four cells and four digits.
        // Their shared restricted candidate cannot occur in both sets, so for
        // every other common digit at least one set must contain it.
        struct Als {
            cells: Vec<usize>,
            cellbits: u128,
            digits: u16,
        }
        let mut als = Vec::<Als>::new();
        let mut seen = std::collections::HashSet::<u128>::new();
        for unit in units() {
            let u = unit.cells();
            for choice in 1u16..512 {
                let n = choice.count_ones();
                if n > 8 {
                    continue;
                }
                let cells: Vec<_> = (0..9)
                    .filter(|&p| choice & (1 << p) != 0)
                    .map(|p| u[p])
                    .collect();
                if cells
                    .iter()
                    .any(|&i| masks[i].count_ones() < 2 || masks[i].count_ones() > 9)
                {
                    continue;
                }
                let digits = cells.iter().fold(0u16, |a, &i| a | masks[i]);
                if digits.count_ones() != n + 1 {
                    continue;
                }
                let cellbits = cells.iter().fold(0u128, |a, &i| a | (1u128 << i));
                if seen.insert(cellbits) {
                    als.push(Als {
                        cells,
                        cellbits,
                        digits,
                    });
                }
            }
        }
        for a in 0..als.len() {
            for b in a + 1..als.len() {
                let (a, b) = (&als[a], &als[b]);
                if a.cells.len() + b.cells.len() != 4
                    || a.cellbits & b.cellbits != 0
                    || (a.digits | b.digits).count_ones() != 4
                {
                    continue;
                }
                let common = a.digits & b.digits;
                for digit in 1..=9 {
                    let r = 1u16 << digit;
                    if common & r == 0 {
                        continue;
                    }
                    let ar: Vec<_> = a
                        .cells
                        .iter()
                        .copied()
                        .filter(|&i| masks[i] & r != 0)
                        .collect();
                    let br: Vec<_> = b
                        .cells
                        .iter()
                        .copied()
                        .filter(|&i| masks[i] & r != 0)
                        .collect();
                    if !ar.iter().all(|&i| br.iter().all(|&j| sees(i, j))) {
                        continue;
                    }
                    for d in 1..=9 {
                        let z = 1u16 << d;
                        if z == r || common & z == 0 {
                            continue;
                        }
                        let holders: Vec<_> = a
                            .cells
                            .iter()
                            .chain(&b.cells)
                            .copied()
                            .filter(|&i| masks[i] & z != 0)
                            .collect();
                        let mut h = EliminationHint {
                            technique: "WXYZ-Wing",
                            unit: Unit::Row(a.cells[0] / 9),
                            second_unit: None,
                            fins: [false; 81],
                            als_groups: [0; 81],
                            als_links: [0; 5],
                            cover_lines: 0,
                            structure: [0; 81],
                            removals: [0; 81],
                        };
                        for &i in a.cells.iter().chain(&b.cells) {
                            h.structure[i] = masks[i];
                        }
                        for i in 0..81 {
                            if (a.cellbits | b.cellbits) & (1u128 << i) == 0
                                && holders.iter().all(|&j| sees(i, j))
                            {
                                h.removals[i] = masks[i] & z;
                            }
                        }
                        if h.removals.iter().any(|&m| m != 0) && visit(h) {
                            return;
                        }
                    }
                }
            }
        }
        // Four-cell WXYZ subset: one 4-candidate pivot sees three subset wings.
        // Exhaustively prove that no peer-consistent assignment exists without z;
        // only cells seeing every z occurrence may then lose z.
        for pivot in 0..81 {
            if masks[pivot].count_ones() != 4 {
                continue;
            }
            let wings: Vec<_> = (0..81)
                .filter(|&i| {
                    sees(pivot, i)
                        && (2..=3).contains(&masks[i].count_ones())
                        && masks[i] & !masks[pivot] == 0
                })
                .collect();
            for x in 0..wings.len() {
                for y in x + 1..wings.len() {
                    for zidx in y + 1..wings.len() {
                        let cells = [pivot, wings[x], wings[y], wings[zidx]];
                        for digit in 1..=9 {
                            let z = 1u16 << digit;
                            if masks[pivot] & z == 0 {
                                continue;
                            }
                            if wing_without_z(&cells, masks, z, 0, &mut [0; 4]) {
                                continue;
                            }
                            let holders: Vec<_> = cells
                                .iter()
                                .copied()
                                .filter(|&i| masks[i] & z != 0)
                                .collect();
                            let mut h = EliminationHint {
                                technique: "WXYZ-Wing",
                                unit: Unit::Row(pivot / 9),
                                second_unit: None,
                                fins: [false; 81],
                                als_groups: [0; 81],
                                als_links: [0; 5],
                                cover_lines: 0,
                                structure: [0; 81],
                                removals: [0; 81],
                            };
                            for i in cells {
                                h.structure[i] = masks[i];
                            }
                            for i in 0..81 {
                                if !cells.contains(&i) && holders.iter().all(|&j| sees(i, j)) {
                                    h.removals[i] = masks[i] & z;
                                }
                            }
                            if h.removals.iter().any(|&m| m != 0) && visit(h) {
                                return;
                            }
                        }
                    }
                }
            }
        }
        // General disjoint ALS pair: a restricted common candidate r forces
        // either set to contain each of their other shared digits.
        for a in 0..als.len() {
            for b in a + 1..als.len() {
                let (a, b) = (&als[a], &als[b]);
                if a.cellbits & b.cellbits != 0 {
                    continue;
                }
                let common = a.digits & b.digits;
                if common.count_ones() < 2 {
                    continue;
                }
                for digit in 1..=9 {
                    let r = 1u16 << digit;
                    if common & r == 0 {
                        continue;
                    }
                    let ar: Vec<_> = a
                        .cells
                        .iter()
                        .copied()
                        .filter(|&i| masks[i] & r != 0)
                        .collect();
                    let br: Vec<_> = b
                        .cells
                        .iter()
                        .copied()
                        .filter(|&i| masks[i] & r != 0)
                        .collect();
                    if !ar.iter().all(|&i| br.iter().all(|&j| sees(i, j))) {
                        continue;
                    }
                    for d in 1..=9 {
                        let z = 1u16 << d;
                        if z == r || common & z == 0 {
                            continue;
                        }
                        let holders: Vec<_> = a
                            .cells
                            .iter()
                            .chain(&b.cells)
                            .copied()
                            .filter(|&i| masks[i] & z != 0)
                            .collect();
                        let mut h = EliminationHint {
                            technique: "Almost Locked Sets",
                            unit: Unit::Row(a.cells[0] / 9),
                            second_unit: None,
                            fins: [false; 81],
                            als_groups: [0; 81],
                            als_links: [0; 5],
                            cover_lines: 0,
                            structure: [0; 81],
                            removals: [0; 81],
                        };
                        h.als_links[0] = r;
                        for (group, set) in [a, b].into_iter().enumerate() {
                            for &i in &set.cells {
                                h.structure[i] = masks[i];
                                h.als_groups[i] = group as u8 + 1;
                            }
                        }
                        for i in 0..81 {
                            if (a.cellbits | b.cellbits) & (1u128 << i) == 0
                                && holders.iter().all(|&j| sees(i, j))
                            {
                                h.removals[i] = masks[i] & z;
                            }
                        }
                        if h.removals.iter().any(|&m| m != 0) && visit(h) {
                            return;
                        }
                    }
                }
            }
        }
        // ALS chains: adjacent ALSs have a restricted common candidate and
        // consecutive link digits differ. If endpoint digit z is absent in the
        // first ALS, the links force z into the last. All sets are disjoint.
        let mut peers = [0u128; 81];
        let mut digit_cells = [0u128; 10];
        for i in 0..81 {
            for j in 0..81 {
                if sees(i, j) {
                    peers[i] |= 1u128 << j;
                }
            }
            for d in 1..=9 {
                if masks[i] & (1 << d) != 0 {
                    digit_cells[d] |= 1u128 << i;
                }
            }
        }
        let mut visible = vec![[0u128; 10]; als.len()];
        let mut occurrences = vec![[0u128; 10]; als.len()];
        for (ai, a) in als.iter().enumerate() {
            for d in 1..=9 {
                if a.digits & (1 << d) == 0 {
                    continue;
                }
                let mut v = u128::MAX;
                for &i in &a.cells {
                    if masks[i] & (1 << d) != 0 {
                        occurrences[ai][d] |= 1u128 << i;
                        v &= peers[i];
                    }
                }
                visible[ai][d] = v;
            }
        }
        let mut graph = vec![Vec::<(usize, u16)>::new(); als.len()];
        for i in 0..als.len() {
            for j in i + 1..als.len() {
                if als[i].cellbits & als[j].cellbits != 0 {
                    continue;
                }
                let common = als[i].digits & als[j].digits;
                let mut links = 0u16;
                for d in 1..=9 {
                    let bit = 1u16 << d;
                    if common & bit != 0 && occurrences[i][d] & !visible[j][d] == 0 {
                        links |= bit;
                    }
                }
                if links != 0 {
                    graph[i].push((j, links));
                    graph[j].push((i, links));
                }
            }
        }
        for start in 0..als.len() {
            for z in 1..=9 {
                let zbit = 1u16 << z;
                if als[start].digits & zbit == 0 {
                    continue;
                }
                let targets = digit_cells[z] & visible[start][z];
                if targets == 0 {
                    continue;
                }
                for &(mid, link1) in &graph[start] {
                    let link1 = link1 & !zbit;
                    for d in 1..=9 {
                        let r1 = 1u16 << d;
                        if link1 & r1 == 0 {
                            continue;
                        }
                        for &(end, link2) in &graph[mid] {
                            if als[start].cellbits & als[end].cellbits != 0
                                || als[end].digits & zbit == 0
                            {
                                continue;
                            }
                            if link2 & !r1 & !zbit == 0 {
                                continue;
                            }
                            let targets = targets
                                & visible[end][z]
                                & !(als[start].cellbits | als[mid].cellbits | als[end].cellbits);
                            if targets == 0 {
                                continue;
                            }
                            let mut h = EliminationHint {
                                technique: "ALS-Chain",
                                unit: Unit::Row(als[start].cells[0] / 9),
                                second_unit: None,
                                fins: [false; 81],
                                als_groups: [0; 81],
                                als_links: [0; 5],
                                cover_lines: 0,
                                structure: [0; 81],
                                removals: [0; 81],
                            };
                            let r2 = link2 & !r1 & !zbit;
                            h.als_links[0] = r1;
                            h.als_links[1] = r2 & r2.wrapping_neg();
                            for (group, &a) in [start, mid, end].iter().enumerate() {
                                for &i in &als[a].cells {
                                    h.structure[i] = masks[i];
                                    h.als_groups[i] = group as u8 + 1;
                                }
                            }
                            for i in 0..81 {
                                if targets & (1u128 << i) != 0 {
                                    h.removals[i] = zbit;
                                }
                            }
                            if visit(h) {
                                return;
                            }
                        }
                    }
                }
            }
        }
        // Four-ALS paths, checked only after all shorter paths.
        for start in 0..als.len() {
            for z in 1..=9 {
                let zbit = 1u16 << z;
                if als[start].digits & zbit == 0 {
                    continue;
                }
                let targets = digit_cells[z] & visible[start][z];
                if targets == 0 {
                    continue;
                }
                for &(a, link1) in &graph[start] {
                    let first = link1 & !zbit;
                    if first == 0 {
                        continue;
                    }
                    for &(b, link2) in &graph[a] {
                        let used = als[start].cellbits | als[a].cellbits;
                        if used & als[b].cellbits != 0 {
                            continue;
                        }
                        let used = used | als[b].cellbits;
                        for &(end, link3) in &graph[b] {
                            if used & als[end].cellbits != 0 || als[end].digits & zbit == 0 {
                                continue;
                            }
                            let last = link3 & !zbit;
                            if last == 0 {
                                continue;
                            }
                            let targets = targets & visible[end][z] & !(used | als[end].cellbits);
                            if targets == 0 {
                                continue;
                            }
                            let Some(r2) = (1..=9).map(|d| 1u16 << d).find(|&bit| {
                                link2 & bit != 0 && first & !bit != 0 && last & !bit != 0
                            }) else {
                                continue;
                            };
                            let r1 = first & !r2;
                            let r3 = last & !r2;
                            let mut h = EliminationHint {
                                technique: "ALS-Chain",
                                unit: Unit::Row(als[start].cells[0] / 9),
                                second_unit: None,
                                fins: [false; 81],
                                als_groups: [0; 81],
                                als_links: [0; 5],
                                cover_lines: 0,
                                structure: [0; 81],
                                removals: [0; 81],
                            };
                            h.als_links[0] = r1 & r1.wrapping_neg();
                            h.als_links[1] = r2;
                            h.als_links[2] = r3 & r3.wrapping_neg();
                            for (group, &node) in [start, a, b, end].iter().enumerate() {
                                for &i in &als[node].cells {
                                    h.structure[i] = masks[i];
                                    h.als_groups[i] = group as u8 + 1;
                                }
                            }
                            for i in 0..81 {
                                if targets & (1u128 << i) != 0 {
                                    h.removals[i] = zbit;
                                }
                            }
                            if visit(h) {
                                return;
                            }
                        }
                    }
                }
            }
        }
        // A bounded five/six-set continuation; only endpoint occurrences of z
        // need to see the target. Intermediate link digits alternate.
        fn dfs_chain(
            als: &[Als],
            graph: &[Vec<(usize, u16)>],
            visible: &[[u128; 10]],
            masks: &[u16; 81],
            z: usize,
            targets: u128,
            max_len: usize,
            path: &mut Vec<usize>,
            links_in_path: &mut [u16; 5],
            used: u128,
            prev: u16,
            visit: &mut impl FnMut(EliminationHint) -> bool,
        ) -> bool {
            let zbit = 1u16 << z;
            let current = *path.last().unwrap();
            if path.len() == max_len {
                if als[current].digits & zbit == 0 || prev == zbit {
                    return false;
                }
                let targets = targets & visible[current][z] & !used;
                if targets == 0 {
                    return false;
                }
                let mut h = EliminationHint {
                    technique: "ALS-Chain",
                    unit: Unit::Row(als[path[0]].cells[0] / 9),
                    second_unit: None,
                    fins: [false; 81],
                    als_groups: [0; 81],
                    als_links: [0; 5],
                    cover_lines: 0,
                    structure: [0; 81],
                    removals: [0; 81],
                };
                h.als_links = *links_in_path;
                for (group, &node) in path.iter().enumerate() {
                    for &i in &als[node].cells {
                        h.structure[i] = masks[i];
                        h.als_groups[i] = group as u8 + 1;
                    }
                }
                for i in 0..81 {
                    if targets & (1u128 << i) != 0 {
                        h.removals[i] = zbit;
                    }
                }
                return visit(h);
            }
            for &(next, links) in &graph[current] {
                if als[next].cellbits & used != 0 {
                    continue;
                }
                if path.len() + 1 == max_len
                    && (als[next].digits & zbit == 0
                        || targets & visible[next][z] & !(used | als[next].cellbits) == 0)
                {
                    continue;
                }
                let mut allowed = links & !prev;
                if path.len() == 1 || path.len() + 1 == max_len {
                    allowed &= !zbit;
                }
                while allowed != 0 {
                    let bit = allowed & allowed.wrapping_neg();
                    allowed &= !bit;
                    links_in_path[path.len() - 1] = bit;
                    path.push(next);
                    if dfs_chain(
                        als,
                        graph,
                        visible,
                        masks,
                        z,
                        targets,
                        max_len,
                        path,
                        links_in_path,
                        used | als[next].cellbits,
                        bit,
                        visit,
                    ) {
                        return true;
                    }
                    path.pop();
                }
            }
            false
        }
        for max_len in 5..=6 {
            for start in 0..als.len() {
                for z in 1..=9 {
                    if als[start].digits & (1 << z) == 0 {
                        continue;
                    }
                    let targets = digit_cells[z] & visible[start][z];
                    if targets == 0 {
                        continue;
                    }
                    if dfs_chain(
                        &als,
                        &graph,
                        &visible,
                        masks,
                        z,
                        targets,
                        max_len,
                        &mut vec![start],
                        &mut [0; 5],
                        als[start].cellbits,
                        0,
                        &mut visit,
                    ) {
                        return;
                    }
                }
            }
        }
    }
    pub fn apply_elimination(&mut self, hint: EliminationHint) -> bool {
        if self.logical_hint() != Some(LogicHint::Elimination(hint)) {
            return false;
        }
        for i in 0..81 {
            self.cells[i].candidates &= !hint.removals[i];
        }
        true
    }
    pub fn elimination_visual(&self, hint: EliminationHint, stage: u8) -> HintVisual {
        let mut v = HintVisual::default();
        for i in 0..81 {
            if hint.structure[i] != 0
                && (self.cells[i].candidates & hint.structure[i] != 0 || hint.als_groups[i] != 0)
            {
                v.cells[i] = if stage < 3 {
                    HintCellStyle::BlueOutline
                } else if hint.als_groups[i] != 0 {
                    match hint.als_groups[i] {
                        1 => HintCellStyle::AlsBlue,
                        2 => HintCellStyle::AlsGreen,
                        3 => HintCellStyle::AlsOrange,
                        4 => HintCellStyle::AlsYellow,
                        _ => HintCellStyle::AlsRed,
                    }
                } else if hint.fins[i]
                    || (hint.technique == "XY-Wing"
                        && hint.structure[i] & hint.removals.iter().fold(0, |a, b| a | b) == 0)
                {
                    HintCellStyle::GreenOutline
                } else {
                    HintCellStyle::GreenFill
                };
                if stage >= 3 {
                    v.als_groups[i] = hint.als_groups[i];
                    v.highlights[i] = hint.structure[i] & self.cells[i].candidates;
                }
            }
        }
        if stage >= 3 {
            if !matches!(
                hint.technique,
                "XY-Wing" | "XYZ-Wing" | "WXYZ-Wing" | "Almost Locked Sets" | "ALS-Chain"
            ) {
                for i in hint.unit.cells() {
                    v.area[i] = true;
                }
                if let Some(u) = hint.second_unit {
                    for i in u.cells() {
                        v.area[i] = true;
                    }
                }
            }
            if matches!(
                hint.technique,
                "X-Wing"
                    | "Swordfish"
                    | "Jellyfish"
                    | "Squirmbag"
                    | "Finned X-Wing"
                    | "Finned Swordfish"
                    | "Finned Jellyfish"
                    | "Finned Squirmbag"
            ) {
                for i in 0..81 {
                    if hint.structure[i] == 0 {
                        continue;
                    }
                    let (base, cover, pos) = match hint.unit {
                        Unit::Row(_) => (Unit::Row(i / 9), Unit::Column(i % 9), i % 9),
                        Unit::Column(_) => (Unit::Column(i % 9), Unit::Row(i / 9), i / 9),
                        Unit::Block(_) => continue,
                    };
                    for cell in base.cells() {
                        v.area[cell] = true;
                    }
                    if hint.cover_lines == 0 || hint.cover_lines & (1 << pos) != 0 {
                        for cell in cover.cells() {
                            v.area[cell] = true;
                        }
                    }
                }
            }
            for i in 0..81 {
                v.removals[i] = hint.removals[i] & self.cells[i].candidates;
                if v.removals[i] != 0 && hint.structure[i] == 0 {
                    v.cells[i] = HintCellStyle::OrangeOutline;
                }
            }
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expert_fish_both_orientations_digit_nine() {
        let board = Board::parse(&"0".repeat(81)).unwrap();
        for transpose in [false, true] {
            let index = |i: usize| if transpose { (i % 9) * 9 + i / 9 } else { i };
            for (name, cells, target) in [
                ("Squirmbag", vec![0, 1, 10, 11, 20, 21, 30, 31, 36, 40], 72),
                ("Finned Swordfish", vec![0, 2, 6, 27, 30, 57, 60], 9),
                ("Finned Jellyfish", vec![0, 2, 8, 27, 30, 48, 51, 69, 71], 9),
                (
                    "Finned Squirmbag",
                    vec![0, 2, 8, 27, 30, 48, 49, 67, 69, 78, 80],
                    9,
                ),
            ] {
                let mut masks = [0; 81];
                for i in cells {
                    masks[index(i)] = 1 << 9;
                }
                masks[index(target)] = 1 << 9;
                let mut found = None;
                board.visit_hard_eliminations(&masks, |h| {
                    if h.technique == name && h.removals[index(target)] != 0 {
                        found = Some(h);
                        true
                    } else {
                        false
                    }
                });
                let h = found.unwrap_or_else(|| panic!("{name}, transpose={transpose}"));
                assert_eq!(h.removals[index(target)], 1 << 9);
                if name.starts_with("Finned") {
                    assert!(h.fins.iter().any(|&b| b));
                }
            }
        }
    }
    #[test]
    fn wxyz_als_proof_and_common_peers() {
        let board = Board::parse(&"0".repeat(81)).unwrap();
        let mut masks = [0; 81];
        masks[0] = (1 << 1) | (1 << 2);
        masks[1] = (1 << 2) | (1 << 3);
        masks[9] = (1 << 1) | (1 << 4);
        masks[18] = (1 << 3) | (1 << 4);
        masks[19] = (1 << 3) | (1 << 5);
        masks[80] = 1 << 3;
        let mut found = None;
        board.visit_hard_eliminations(&masks, |h| {
            if h.technique == "WXYZ-Wing" && h.removals[19] & (1 << 3) != 0 {
                found = Some(h);
                true
            } else {
                false
            }
        });
        let h = found.expect("ALS-based WXYZ");
        let cells: Vec<_> = (0..81).filter(|&i| h.structure[i] != 0).collect();
        let cells: [usize; 4] = cells.try_into().unwrap();
        assert!(!wing_without_z(
            &cells,
            &h.structure,
            1 << 3,
            0,
            &mut [0; 4]
        ));
        assert!(cells
            .iter()
            .filter(|&&i| h.structure[i] & (1 << 3) != 0)
            .all(|&i| sees(i, 19)));
        assert_eq!(h.removals[80], 0);
    }
    #[test]
    fn wing_assignment_checks_peer_constraints() {
        let cells = [0, 3, 27, 6];
        let mut masks = [0; 81];
        masks[0] = (1 << 1) | (1 << 2) | (1 << 3) | (1 << 9);
        masks[3] = (1 << 1) | (1 << 2);
        masks[27] = (1 << 1) | (1 << 3);
        masks[6] = (1 << 1) | (1 << 9);
        assert!(!wing_without_z(&cells, &masks, 1 << 1, 0, &mut [0; 4]));
        masks[6] |= 1 << 3;
        assert!(wing_without_z(&cells, &masks, 1 << 1, 0, &mut [0; 4]));
    }
    #[test]
    fn hard_patterns_cover_orientations_and_digit_nine() {
        let board = Board::parse(&"0".repeat(81)).unwrap();
        for transpose in [false, true] {
            let index = |i: usize| if transpose { (i % 9) * 9 + i / 9 } else { i };
            for (name, corners, target) in [
                ("Swordfish", vec![1, 4, 31, 34, 55, 61], 73),
                ("Jellyfish", vec![0, 2, 20, 22, 40, 42, 54, 60], 72),
                ("Finned X-Wing", vec![1, 2, 5, 37, 41], 10),
            ] {
                let mut masks = [0; 81];
                for i in corners {
                    masks[index(i)] = 1 << 9;
                }
                masks[index(target)] = 1 << 9;
                if name == "Finned X-Wing" {
                    masks[index(73)] = 1 << 9;
                }
                let mut found = None;
                board.visit_hard_eliminations(&masks, |h| {
                    if h.technique == name && h.removals[index(target)] != 0 {
                        found = Some(h);
                        true
                    } else {
                        false
                    }
                });
                let h = found.unwrap_or_else(|| panic!("{name} transpose={transpose}"));
                assert_eq!(h.removals[index(target)], 1 << 9);
                if name == "Finned X-Wing" {
                    assert_eq!(h.removals[index(73)], 0);
                }
            }
        }
        let mut invalid_fins = [0; 81];
        for i in [1, 2, 5, 8, 37, 41, 10] {
            invalid_fins[i] = 1 << 9;
        }
        let mut invalid_found = false;
        board.visit_hard_eliminations(&invalid_fins, |h| {
            if h.technique == "Finned X-Wing" && h.removals[10] != 0 {
                invalid_found = true;
                true
            } else {
                false
            }
        });
        assert!(
            !invalid_found,
            "fins in different blocks must not justify elimination"
        );
        let mut masks = [0; 81];
        masks[0] = (1 << 1) | (1 << 2);
        masks[3] = (1 << 1) | (1 << 9);
        masks[27] = (1 << 2) | (1 << 9);
        masks[30] = (1 << 3) | (1 << 4) | (1 << 9);
        masks[80] = 1 << 9;
        let mut found = None;
        board.visit_hard_eliminations(&masks, |h| {
            if h.technique == "XY-Wing" {
                found = Some(h);
                true
            } else {
                false
            }
        });
        let h = found.expect("XY-Wing");
        assert_eq!(h.removals[30], 1 << 9);
        assert_eq!(h.removals[80], 0);
    }
    #[test]
    fn x_wing_covers_both_orientations_and_digit_nine() {
        let board = Board::parse(&"0".repeat(81)).unwrap();
        for (corners, target, expected_unit) in [
            ([1, 5, 37, 41], 73, Unit::Row(0)),
            ([1, 5, 37, 41], 8, Unit::Column(1)),
        ] {
            let mut masks = [0; 81];
            for i in corners {
                masks[i] = 1 << 9;
            }
            masks[target] = 1 << 9;
            let mut found = None;
            board.visit_eliminations(&masks, |h| {
                if h.technique == "X-Wing" {
                    found = Some(h);
                    true
                } else {
                    false
                }
            });
            let h = found.expect("X-Wing");
            assert_eq!(h.unit, expected_unit);
            assert_eq!(h.removals[target], 1 << 9);
            for i in corners {
                assert_eq!(h.structure[i], 1 << 9);
                assert_eq!(h.removals[i], 0);
            }
        }
    }
    #[test]
    fn pairs_include_digit_nine_and_do_not_use_partial_notes_as_proof() {
        let board = Board::parse(
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let mut masks = [0; 81];
        masks[0] = (1 << 1) | (1 << 8) | (1 << 9);
        masks[1] = (1 << 2) | (1 << 8) | (1 << 9);
        masks[2] = (1 << 1) | (1 << 2);
        let mut first = None;
        board.visit_eliminations(&masks, |h| {
            first = Some(h);
            true
        });
        let h = first.unwrap();
        assert_eq!(h.technique, "Hidden Pair");
        assert_eq!(h.structure[0], (1 << 8) | (1 << 9));
        assert_eq!(h.structure[1], (1 << 8) | (1 << 9));
        assert_eq!(h.removals[0], 1 << 1);
        assert_eq!(h.removals[1], 1 << 2);
        masks = [0; 81];
        masks[0] = (1 << 8) | (1 << 9);
        masks[1] = masks[0];
        masks[2] = (1 << 1) | (1 << 8);
        first = None;
        board.visit_eliminations(&masks, |h| {
            first = Some(h);
            true
        });
        let h = first.unwrap();
        assert_eq!(h.technique, "Naked Pair");
        assert_eq!(h.removals[2], 1 << 8);
        // On an empty board, deleting arbitrary notes cannot logically create a pair.
        let mut incomplete = board;
        incomplete.set_auto_candidates(true);
        for d in 1..=7 {
            assert!(incomplete.toggle_candidate(0, d));
            assert!(incomplete.toggle_candidate(1, d));
        }
        assert_eq!(incomplete.logical_hint(), None);
    }
    #[test]
    fn pair_search_covers_rows_columns_and_blocks() {
        let board = Board::parse(&"0".repeat(81)).unwrap();
        for (unit, a, b, target) in [
            (Unit::Row(0), 0, 1, 2),
            (Unit::Column(0), 0, 9, 18),
            (Unit::Block(0), 0, 10, 20),
        ] {
            let mut masks = [0; 81];
            masks[a] = (1 << 8) | (1 << 9) | (1 << 1);
            masks[b] = (1 << 8) | (1 << 9) | (1 << 2);
            let mut h = None;
            board.visit_eliminations(&masks, |v| {
                h = Some(v);
                true
            });
            let pair = h.unwrap();
            assert_eq!((pair.technique, pair.unit), ("Hidden Pair", unit));
            masks = [0; 81];
            masks[a] = (1 << 8) | (1 << 9);
            masks[b] = masks[a];
            masks[target] = (1 << 1) | (1 << 8);
            h = None;
            board.visit_eliminations(&masks, |v| {
                h = Some(v);
                true
            });
            let pair = h.unwrap();
            assert_eq!((pair.technique, pair.unit), ("Naked Pair", unit));
            assert_eq!(pair.removals[target], 1 << 8);
        }
    }
    #[test]
    fn elimination_preview_is_checked_and_chain_reconstruction_is_safe() {
        let mut board = Board::parse(
            "500132008000000600030700010000000005000480070000000800820301700950000200700004000",
        )
        .unwrap();
        let solution = Solution::parse_for(
            &board,
            "567132498281549637439768512348617925692485173175923864824391756953876241716254389",
        )
        .unwrap();
        board.set_auto_candidates(true);
        let mut eliminations = 0;
        for _ in 0..150 {
            if board.is_solved() {
                break;
            }
            match board.logical_hint().expect("hint") {
                LogicHint::Single(h) => {
                    assert_eq!(solution.digit(h.index), Some(h.digit));
                    assert!(board.set_value(h.index, h.digit));
                }
                LogicHint::Elimination(h) => {
                    let snapshot = board.clone();
                    let mut forged = h;
                    forged.removals[0] ^= 2;
                    assert!(!board.apply_elimination(forged));
                    assert_eq!(board, snapshot);
                    for i in 0..81 {
                        assert_eq!(h.removals[i] & (1 << solution.digit(i).unwrap()), 0);
                    }
                    assert!(board.apply_elimination(h));
                    assert!(!board.apply_elimination(h));
                    eliminations += 1;
                }
            }
        }
        assert!(board.is_solved());
        assert!(eliminations > 0);
    }
}
