//! MD-3: Cell list for O(N) pairwise force computation.
//!
//! Divides the periodic box into cells of side >= r_cut. Each particle checks
//! only 9 neighboring cells (2D). O(N) build, O(N * avg_neighbors) iteration.
//!
//! Requires PBC (sim_box). Falls back to brute force without PBC.
//! Axiom 7: only pairs within r_cut interact (distance attenuation cutoff).

use crate::blueprint::equations::pbc;

/// Linked-list cell structure for spatial pair iteration.
pub struct CellList {
    /// Grid dimensions [nx, ny].
    grid_dims: [usize; 2],
    /// Cell side length (>= r_cut).
    _cell_size: f32,
    /// Cutoff radius squared.
    r_cut_sq: f32,
    /// Box dimensions for PBC wrapping.
    box_lengths: [f32; 2],
    /// Head of linked list per cell. `head[cell] = first particle index`, or `EMPTY`.
    head: Vec<u16>,
    /// Next pointer per particle. `next[i] = next particle in same cell`, or `EMPTY`.
    next: Vec<u16>,
}

const EMPTY: u16 = u16::MAX;

impl CellList {
    /// Build cell list from positions. O(N).
    ///
    /// Returns `None` if box is too small (< 3 cells per dimension).
    pub fn build(
        positions: &[[f32; 2]],
        count: usize,
        box_lengths: [f32; 2],
        r_cut: f32,
    ) -> Option<Self> {
        let n = count.min(positions.len());
        // Grid: cells of side >= r_cut so neighbors within r_cut are in adjacent cells.
        let nx = (box_lengths[0] / r_cut).floor() as usize;
        let ny = (box_lengths[1] / r_cut).floor() as usize;
        // Need >= 3 cells per dimension to avoid self-image interaction.
        if nx < 3 || ny < 3 {
            return None;
        }
        let cell_size_x = box_lengths[0] / nx as f32;
        let cell_size_y = box_lengths[1] / ny as f32;
        let cell_size = cell_size_x.max(cell_size_y);

        let n_cells = nx * ny;
        let mut head = vec![EMPTY; n_cells];
        let mut next = vec![EMPTY; n];

        // Insert each particle at head of its cell's list.
        for i in 0..n {
            let cx = ((positions[i][0] / cell_size_x).floor() as usize).min(nx - 1);
            let cy = ((positions[i][1] / cell_size_y).floor() as usize).min(ny - 1);
            let cell = cy * nx + cx;
            next[i] = head[cell];
            head[cell] = i as u16;
        }

        Some(Self {
            grid_dims: [nx, ny],
            _cell_size: cell_size,
            r_cut_sq: r_cut * r_cut,
            box_lengths,
            head,
            next,
        })
    }

    /// Iterate all unique pairs (i < j) within r_cut. Calls `f(i, j, dx, dy)`.
    ///
    /// Uses half-shell neighbor iteration: 5 neighbor offsets (self + 4 forward)
    /// to count each pair exactly once. Newton 3 is the caller's responsibility.
    /// Displacement (dx, dy) uses minimum image convention.
    pub fn for_each_pair(
        &self,
        positions: &[[f32; 2]],
        mut f: impl FnMut(usize, usize, f32, f32),
    ) {
        let [nx, ny] = self.grid_dims;
        let bl = self.box_lengths;
        let r_cut_sq = self.r_cut_sq;

        // Half-shell offsets: self + 4 forward neighbors.
        // (0,0)=self, (1,0), (nx-1,1)=(-1,1), (0,1), (1,1).
        let offsets: [(i32, i32); 5] = [(0, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

        for cy in 0..ny {
            for cx in 0..nx {
                let cell_a = cy * nx + cx;

                for &(dcx, dcy) in &offsets {
                    let ncx = (cx as i32 + dcx).rem_euclid(nx as i32) as usize;
                    let ncy = (cy as i32 + dcy).rem_euclid(ny as i32) as usize;
                    let cell_b = ncy * nx + ncx;
                    let same_cell = cell_a == cell_b;

                    // Iterate pairs between cell_a and cell_b.
                    let mut i = self.head[cell_a];
                    while i != EMPTY {
                        let ii = i as usize;
                        // For same cell: start j after i. For neighbor: start at head.
                        let mut j = if same_cell {
                            self.next[ii]
                        } else {
                            self.head[cell_b]
                        };
                        while j != EMPTY {
                            let jj = j as usize;
                            let d = pbc::minimum_image_2d(
                                positions[ii],
                                positions[jj],
                                bl,
                            );
                            let r_sq = d[0] * d[0] + d[1] * d[1];
                            if r_sq < r_cut_sq {
                                f(ii, jj, d[0], d[1]);
                            }
                            j = self.next[jj];
                        }
                        i = self.next[ii];
                    }
                }
            }
        }
    }
}

// ─── 3D Cell List (MD-7) ────────────────────────────────────────────────────

/// 3D cell list for f64 positions. 27 neighbor cells (3^3).
pub struct CellList3D {
    grid_dims: [usize; 3],
    _cell_size: [f64; 3],
    r_cut_sq: f64,
    box_lengths: [f64; 3],
    head: Vec<u16>,
    next: Vec<u16>,
}

impl CellList3D {
    /// Build 3D cell list. Returns None if box too small (< 3 cells per dim).
    pub fn build(
        positions: &[[f64; 3]],
        count: usize,
        box_lengths: [f64; 3],
        r_cut: f64,
    ) -> Option<Self> {
        let n = count.min(positions.len());
        let nx = (box_lengths[0] / r_cut).floor() as usize;
        let ny = (box_lengths[1] / r_cut).floor() as usize;
        let nz = (box_lengths[2] / r_cut).floor() as usize;
        if nx < 3 || ny < 3 || nz < 3 {
            return None;
        }
        let cs = [
            box_lengths[0] / nx as f64,
            box_lengths[1] / ny as f64,
            box_lengths[2] / nz as f64,
        ];

        let n_cells = nx * ny * nz;
        let mut head = vec![EMPTY; n_cells];
        let mut next = vec![EMPTY; n];

        for i in 0..n {
            let cx = ((positions[i][0] / cs[0]).floor() as usize).min(nx - 1);
            let cy = ((positions[i][1] / cs[1]).floor() as usize).min(ny - 1);
            let cz = ((positions[i][2] / cs[2]).floor() as usize).min(nz - 1);
            let cell = cz * ny * nx + cy * nx + cx;
            // Linked list: insert at head of cell chain
            next[i] = head[cell];
            head[cell] = i as u16;
        }

        Some(Self {
            grid_dims: [nx, ny, nz],
            _cell_size: cs,
            r_cut_sq: r_cut * r_cut,
            box_lengths,
            head,
            next,
        })
    }

    /// Iterate all unique pairs within r_cut. Calls `f(i, j, dx, dy, dz)`.
    ///
    /// Half-shell: 14 offsets (self + 13 forward) for 3D.
    pub fn for_each_pair(
        &self,
        positions: &[[f64; 3]],
        mut f: impl FnMut(usize, usize, f64, f64, f64),
    ) {
        let [nx, ny, nz] = self.grid_dims;
        let bl = self.box_lengths;
        let r_cut_sq = self.r_cut_sq;

        // Half-shell 3D: 14 offsets (self + 13 forward neighbors)
        // Forward = (dcz > 0) || (dcz == 0 && dcy > 0) || (dcz == 0 && dcy == 0 && dcx > 0)
        let mut offsets = Vec::with_capacity(14);
        for dcz in -1i32..=1 {
            for dcy in -1i32..=1 {
                for dcx in -1i32..=1 {
                    if dcz > 0 || (dcz == 0 && dcy > 0) || (dcz == 0 && dcy == 0 && dcx >= 0) {
                        offsets.push((dcx, dcy, dcz));
                    }
                }
            }
        }

        for cz in 0..nz {
            for cy in 0..ny {
                for cx in 0..nx {
                    let cell_a = cz * ny * nx + cy * nx + cx;

                    for &(dcx, dcy, dcz) in &offsets {
                        let ncx = (cx as i32 + dcx).rem_euclid(nx as i32) as usize;
                        let ncy = (cy as i32 + dcy).rem_euclid(ny as i32) as usize;
                        let ncz = (cz as i32 + dcz).rem_euclid(nz as i32) as usize;
                        let cell_b = ncz * ny * nx + ncy * nx + ncx;
                        let same_cell = cell_a == cell_b;

                        let mut i = self.head[cell_a];
                        while i != EMPTY {
                            let ii = i as usize;
                            let mut j = if same_cell {
                                self.next[ii]
                            } else {
                                self.head[cell_b]
                            };
                            while j != EMPTY {
                                let jj = j as usize;
                                let d = pbc::minimum_image_3d(positions[ii], positions[jj], bl);
                                let r_sq = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
                                if r_sq < r_cut_sq {
                                    f(ii, jj, d[0], d[1], d[2]);
                                }
                                j = self.next[jj];
                            }
                            i = self.next[ii];
                        }
                    }
                }
            }
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_returns_none_for_small_box() {
        let pos = [[0.5, 0.5]];
        // Box 1.0×1.0, r_cut=0.5 → 2 cells per dim → too few (<3)
        assert!(CellList::build(&pos, 1, [1.0, 1.0], 0.5).is_none());
    }

    #[test]
    fn build_succeeds_for_adequate_box() {
        let pos = [[0.5, 0.5]];
        let cl = CellList::build(&pos, 1, [3.0, 3.0], 0.5);
        assert!(cl.is_some());
        let cl = cl.unwrap();
        assert!(cl.grid_dims[0] >= 3);
        assert!(cl.grid_dims[1] >= 3);
    }

    #[test]
    fn single_particle_no_pairs() {
        let pos = [[1.0, 1.0]];
        let cl = CellList::build(&pos, 1, [5.0, 5.0], 1.0).unwrap();
        let mut count = 0;
        cl.for_each_pair(&pos, |_, _, _, _| count += 1);
        assert_eq!(count, 0, "single particle → no pairs");
    }

    #[test]
    fn two_close_particles_found() {
        let pos = [[1.0, 1.0], [1.3, 1.0]]; // distance 0.3 < r_cut=1.0
        let cl = CellList::build(&pos, 2, [5.0, 5.0], 1.0).unwrap();
        let mut count = 0;
        cl.for_each_pair(&pos, |_, _, _, _| count += 1);
        assert_eq!(count, 1, "close pair found");
    }

    #[test]
    fn two_distant_particles_skipped() {
        let pos = [[0.5, 0.5], [4.0, 4.0]]; // distance > r_cut
        let cl = CellList::build(&pos, 2, [5.0, 5.0], 1.0).unwrap();
        let mut count = 0;
        cl.for_each_pair(&pos, |_, _, _, _| count += 1);
        assert_eq!(count, 0, "far pair skipped");
    }

    #[test]
    fn pbc_wrap_detects_boundary_pair() {
        // Particles at opposite edges: (0.1, 2.5) and (4.9, 2.5) in box 5×5
        // Naive distance = 4.8, but minimum image = 0.2 < r_cut=1.0
        let pos = [[0.1, 2.5], [4.9, 2.5]];
        let cl = CellList::build(&pos, 2, [5.0, 5.0], 1.0).unwrap();
        let mut count = 0;
        let mut found_dx = 0.0f32;
        cl.for_each_pair(&pos, |_, _, dx, _| {
            count += 1;
            found_dx = dx;
        });
        assert_eq!(count, 1, "PBC boundary pair found");
        assert!(found_dx.abs() < 1.0, "minimum image dx={found_dx}");
    }

    #[test]
    fn all_pairs_found_matches_brute_force() {
        // 20 particles in a 5×5 box, r_cut=1.5
        let n = 20;
        let r_cut = 1.5;
        let bl = [5.0, 5.0];
        let mut pos = [[0.0f32; 2]; 20];
        for i in 0..n {
            pos[i] = [(i as f32 * 1.17) % bl[0], (i as f32 * 0.83) % bl[1]];
        }

        // Brute force: count pairs within r_cut using minimum image
        let mut brute_pairs = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                let d = pbc::minimum_image_2d(pos[i], pos[j], bl);
                let r_sq = d[0] * d[0] + d[1] * d[1];
                if r_sq < r_cut * r_cut {
                    brute_pairs.push((i, j));
                }
            }
        }

        // Cell list
        let cl = CellList::build(&pos, n, bl, r_cut).unwrap();
        let mut cell_pairs = Vec::new();
        cl.for_each_pair(&pos, |i, j, _, _| {
            let (a, b) = if i < j { (i, j) } else { (j, i) };
            cell_pairs.push((a, b));
        });
        cell_pairs.sort();

        assert_eq!(
            brute_pairs.len(),
            cell_pairs.len(),
            "pair count mismatch: brute={}, cell={}",
            brute_pairs.len(),
            cell_pairs.len(),
        );
        assert_eq!(brute_pairs, cell_pairs, "pair sets differ");
    }

    #[test]
    fn displacement_magnitude_matches_minimum_image() {
        let bl = [5.0, 5.0];
        let pos = [[0.1, 2.5], [4.9, 2.5]];
        let cl = CellList::build(&pos, 2, bl, 1.0).unwrap();

        let mut got_dx = 0.0f32;
        let mut got_dy = 0.0f32;
        cl.for_each_pair(&pos, |_, _, dx, dy| {
            got_dx = dx;
            got_dy = dy;
        });

        let expected = pbc::minimum_image_2d(pos[0], pos[1], bl);
        // Pair order may vary: displacement sign depends on which is i vs j.
        // Check magnitude matches.
        assert!(
            (got_dx.abs() - expected[0].abs()).abs() < 1e-5,
            "|dx|: {} vs {}",
            got_dx.abs(),
            expected[0].abs(),
        );
        assert!(
            (got_dy.abs() - expected[1].abs()).abs() < 1e-5,
            "|dy|: {} vs {}",
            got_dy.abs(),
            expected[1].abs(),
        );
    }

    #[test]
    fn no_self_interaction() {
        let pos = [[2.5, 2.5]];
        let cl = CellList::build(&pos, 1, [5.0, 5.0], 1.0).unwrap();
        let mut count = 0;
        cl.for_each_pair(&pos, |_, _, _, _| count += 1);
        assert_eq!(count, 0, "particle must not interact with itself");
    }

    #[test]
    fn newton3_symmetric_pairs() {
        // Each pair (i,j) appears exactly once — never as both (i,j) and (j,i)
        let n = 15;
        let bl = [5.0, 5.0];
        let mut pos = [[0.0f32; 2]; 15];
        for i in 0..n {
            pos[i] = [(i as f32 * 0.97) % bl[0], (i as f32 * 1.31) % bl[1]];
        }
        let cl = CellList::build(&pos, n, bl, 1.5).unwrap();
        let mut pairs = Vec::new();
        cl.for_each_pair(&pos, |i, j, _, _| {
            let (a, b) = if i < j { (i, j) } else { (j, i) };
            pairs.push((a, b));
        });
        // Check no duplicates
        let len_before = pairs.len();
        pairs.sort();
        pairs.dedup();
        assert_eq!(len_before, pairs.len(), "duplicate pairs detected");
    }

    #[test]
    fn empty_world_no_panic() {
        let pos: [[f32; 2]; 0] = [];
        let cl = CellList::build(&pos, 0, [5.0, 5.0], 1.0).unwrap();
        let mut count = 0;
        cl.for_each_pair(&pos, |_, _, _, _| count += 1);
        assert_eq!(count, 0);
    }
}
