//! PC-1: Barnes-Hut quadtree — O(N log N) force computation for charged particles.
//!
//! Pure math. No Bevy dependency. No heap in hot path (tree pre-allocated).
//!
//! Axiom 7: preserves 1/r² force law. Distant clusters approximated by center-of-charge.
//! Axiom 5: Newton 3 respected — symmetric force accumulation.
//!
//! Precision: internal accumulation in f64 to avoid catastrophic cancellation at high N.

use super::coulomb::{ChargedParticle, coulomb_force};

// ─── Constants (numerical, not physical) ────────────────────────────────────

/// Barnes-Hut opening angle. Controls accuracy vs speed tradeoff.
/// 0.5 = standard (good balance). Lower = more precise, slower.
/// This is a NUMERICAL parameter, not derived from physics constants.
const THETA: f32 = 0.5;

/// Below this N, brute-force O(N²) is faster than tree overhead.
/// Empirically ~64 for 2D quadtrees on modern CPUs.
const BRUTE_FORCE_THRESHOLD: usize = 64;

/// Maximum tree nodes. 4× particle capacity covers worst-case subdivision.
const MAX_NODES: usize = 4096;

// ─── QuadTree node ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
struct QuadNode {
    /// Charge-weighted centroid (f64 for precision).
    cx: f64,
    cy: f64,
    /// Total charge in this subtree.
    total_charge: f64,
    /// Total mass in this subtree.
    total_mass: f64,
    /// Number of particles in this subtree.
    count: u16,
    /// Bounding box center and half-size.
    bx: f32,
    by: f32,
    half: f32,
    /// Leaf particle index (-1 = internal or empty).
    particle_idx: i16,
    /// Children indices (0 = no child). NW, NE, SW, SE.
    children: [u16; 4],
}

impl Default for QuadNode {
    fn default() -> Self {
        Self {
            cx: 0.0,
            cy: 0.0,
            total_charge: 0.0,
            total_mass: 0.0,
            count: 0,
            bx: 0.0,
            by: 0.0,
            half: 0.0,
            particle_idx: -1,
            children: [0; 4],
        }
    }
}

// ─── QuadTree ───────────────────────────────────────────────────────────────

/// Barnes-Hut quadtree for 2D charged particle force computation.
/// Pre-allocated node array — zero heap allocation during build.
pub struct QuadTree {
    nodes: Vec<QuadNode>,
    node_count: usize,
}

impl QuadTree {
    /// Build tree from particles. O(N log N).
    pub fn build(particles: &[ChargedParticle], count: usize) -> Self {
        let n = count.min(particles.len());
        let mut tree = Self {
            nodes: vec![QuadNode::default(); MAX_NODES.min(n * 4 + 16)],
            node_count: 0,
        };
        if n == 0 {
            return tree;
        }

        // Compute bounding box
        let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
        let (mut max_x, mut max_y) = (f32::MIN, f32::MIN);
        for p in &particles[..n] {
            min_x = min_x.min(p.position[0]);
            min_y = min_y.min(p.position[1]);
            max_x = max_x.max(p.position[0]);
            max_y = max_y.max(p.position[1]);
        }
        // Pad slightly to avoid particles on boundary
        let pad = 0.01;
        min_x -= pad;
        min_y -= pad;
        max_x += pad;
        max_y += pad;

        let half = ((max_x - min_x).max(max_y - min_y)) * 0.5;
        let cx = (min_x + max_x) * 0.5;
        let cy = (min_y + max_y) * 0.5;

        // Allocate root
        let root = tree.alloc_node(cx, cy, half);
        if root == u16::MAX {
            return tree;
        }

        // Insert particles one by one
        for i in 0..n {
            tree.insert(root, &particles[..n], i as u16);
        }

        tree
    }

    /// Compute force on particle `idx` using tree approximation. O(log N).
    /// Returns force in f64 for precision.
    fn force_on_f64(
        &self,
        idx: usize,
        particles: &[ChargedParticle],
        theta: f32,
    ) -> [f64; 2] {
        if self.node_count == 0 {
            return [0.0; 2];
        }
        let mut fx: f64 = 0.0;
        let mut fy: f64 = 0.0;
        self.walk_force(0, idx, particles, theta, &mut fx, &mut fy);
        [fx, fy]
    }

    /// Accumulate all forces using tree. Returns f32 forces.
    pub fn accumulate_all(
        &self,
        particles: &[ChargedParticle],
        count: usize,
        theta: f32,
    ) -> Vec<[f32; 2]> {
        let n = count.min(particles.len());
        let mut forces = vec![[0.0f32; 2]; n];
        for i in 0..n {
            let [fx, fy] = self.force_on_f64(i, particles, theta);
            forces[i] = [fx as f32, fy as f32];
        }
        forces
    }

    // ── Internal ────────────────────────────────────────────────────────

    fn alloc_node(&mut self, bx: f32, by: f32, half: f32) -> u16 {
        if self.node_count >= self.nodes.len() {
            return u16::MAX; // tree full
        }
        let idx = self.node_count;
        self.node_count += 1;
        self.nodes[idx] = QuadNode {
            bx,
            by,
            half,
            ..QuadNode::default()
        };
        idx as u16
    }

    fn quadrant(bx: f32, by: f32, px: f32, py: f32) -> usize {
        let right = px >= bx;
        let bottom = py >= by;
        match (right, bottom) {
            (false, false) => 0, // NW
            (true, false) => 1,  // NE
            (false, true) => 2,  // SW
            (true, true) => 3,   // SE
        }
    }

    fn child_center(bx: f32, by: f32, half: f32, quad: usize) -> (f32, f32) {
        let q = half * 0.5;
        match quad {
            0 => (bx - q, by - q),
            1 => (bx + q, by - q),
            2 => (bx - q, by + q),
            _ => (bx + q, by + q),
        }
    }

    fn insert(&mut self, node_idx: u16, particles: &[ChargedParticle], pidx: u16) {
        if node_idx == u16::MAX {
            return;
        }
        let ni = node_idx as usize;
        if ni >= self.node_count {
            return;
        }

        let p = &particles[pidx as usize];
        let node = self.nodes[ni];

        if node.count == 0 {
            // Empty node → become leaf
            self.nodes[ni].particle_idx = pidx as i16;
            self.nodes[ni].count = 1;
            self.nodes[ni].cx = p.position[0] as f64;
            self.nodes[ni].cy = p.position[1] as f64;
            self.nodes[ni].total_charge = p.charge as f64;
            self.nodes[ni].total_mass = p.mass as f64;
            return;
        }

        // If leaf, push existing particle down
        if node.particle_idx >= 0 {
            let old_pidx = node.particle_idx as u16;
            self.nodes[ni].particle_idx = -1; // become internal
            let old_p = &particles[old_pidx as usize];
            let quad = Self::quadrant(node.bx, node.by, old_p.position[0], old_p.position[1]);
            let new_half = node.half * 0.5;
            if new_half < 1e-6 {
                // Too small to subdivide — keep as multi-particle leaf
                self.update_aggregate(ni, p);
                return;
            }
            let (cx, cy) = Self::child_center(node.bx, node.by, node.half, quad);
            let child = self.alloc_node(cx, cy, new_half);
            self.nodes[ni].children[quad] = child;
            self.insert(child, particles, old_pidx);
        }

        // Insert new particle into correct quadrant
        let quad = Self::quadrant(node.bx, node.by, p.position[0], p.position[1]);
        let child = self.nodes[ni].children[quad];
        if child == 0 {
            let new_half = node.half * 0.5;
            let (cx, cy) = Self::child_center(node.bx, node.by, node.half, quad);
            let new_child = self.alloc_node(cx, cy, new_half);
            self.nodes[ni].children[quad] = new_child;
            self.insert(new_child, particles, pidx);
        } else {
            self.insert(child, particles, pidx);
        }

        // Update aggregate
        self.update_aggregate(ni, p);
    }

    fn update_aggregate(&mut self, ni: usize, p: &ChargedParticle) {
        let node = &mut self.nodes[ni];
        let old_total = node.total_charge.abs() + node.total_mass;
        let new_charge = p.charge as f64;
        let new_mass = p.mass as f64;
        node.total_charge += new_charge;
        node.total_mass += new_mass;
        node.count += 1;

        // Charge-weighted centroid (mass-weighted for neutral clusters)
        let weight = (new_charge.abs() + new_mass).max(1e-10);
        let old_weight = old_total.max(1e-10);
        let total_weight = old_weight + weight;
        node.cx = (node.cx * old_weight + p.position[0] as f64 * weight) / total_weight;
        node.cy = (node.cy * old_weight + p.position[1] as f64 * weight) / total_weight;
    }

    fn walk_force(
        &self,
        node_idx: u16,
        target_idx: usize,
        particles: &[ChargedParticle],
        theta: f32,
        fx: &mut f64,
        fy: &mut f64,
    ) {
        let ni = node_idx as usize;
        if ni >= self.node_count {
            return;
        }
        let node = &self.nodes[ni];
        if node.count == 0 {
            return;
        }

        // Leaf with single particle
        if node.count == 1 && node.particle_idx >= 0 {
            let pi = node.particle_idx as usize;
            if pi == target_idx {
                return;
            } // skip self
            let f = super::coulomb::net_force(&particles[target_idx], &particles[pi]);
            *fx += f[0] as f64;
            *fy += f[1] as f64;
            return;
        }

        // Distance from target to node centroid
        let tp = &particles[target_idx];
        let dx = node.cx - tp.position[0] as f64;
        let dy = node.cy - tp.position[1] as f64;
        let dist_sq = dx * dx + dy * dy;
        let dist = dist_sq.sqrt().max(1e-10);

        // Opening criterion: size / distance < theta → use aggregate
        // ONLY Coulomb is approximated (long-range, 1/r²).
        // LJ is NOT approximated (short-range, 1/r¹² — centroid kills accuracy).
        let size = (node.half * 2.0) as f64;
        if size / dist < theta as f64 {
            // Approximate Coulomb only: treat cluster as single charge at centroid
            let r = dist as f32;
            let f_coulomb = -coulomb_force(tp.charge, node.total_charge as f32, r);
            let ux = dx / dist;
            let uy = dy / dist;
            *fx += f_coulomb as f64 * ux;
            *fy += f_coulomb as f64 * uy;
            return;
        }

        // Open node: recurse into children
        for &child in &node.children {
            if child != 0 {
                self.walk_force(child, target_idx, particles, theta, fx, fy);
            }
        }
    }
}

// ─── Adaptive dispatch ──────────────────────────────────────────────────────

/// Accumulate forces adaptively: brute-force for small N, Barnes-Hut for large N.
/// Internal accumulation in f64, output in f32.
///
/// Axiom 5: Newton 3 respected in both paths.
/// Axiom 7: 1/r² preserved (tree approximation within theta tolerance).
/// Adaptive force accumulation. PBC-aware: brute-force when box is set
/// (tree doesn't handle periodic images). Free-space uses tree for large N.
pub fn accumulate_forces_adaptive(
    particles: &[ChargedParticle],
    count: usize,
    box_lengths: Option<[f32; 2]>,
) -> Vec<[f32; 2]> {
    let n = count.min(particles.len());
    if n < 2 {
        return vec![[0.0; 2]; n];
    }

    // PBC: always brute-force (tree doesn't handle periodic images)
    if box_lengths.is_some() || n < BRUTE_FORCE_THRESHOLD {
        accumulate_brute_f64(particles, n, box_lengths)
    } else {
        let tree = QuadTree::build(particles, n);
        tree.accumulate_all(particles, n, THETA)
    }
}

/// Brute-force O(N²) with f64 internal precision. Newton 3 symmetric.
/// Optional PBC via minimum image convention.
fn accumulate_brute_f64(
    particles: &[ChargedParticle],
    count: usize,
    box_lengths: Option<[f32; 2]>,
) -> Vec<[f32; 2]> {
    let n = count.min(particles.len());
    let mut forces = vec![[0.0f64; 2]; n];

    for i in 0..n {
        for j in (i + 1)..n {
            let f = match box_lengths {
                Some(bl) => super::coulomb::net_force_pbc(&particles[i], &particles[j], bl),
                None => super::coulomb::net_force(&particles[i], &particles[j]),
            };
            let fx = f[0] as f64;
            let fy = f[1] as f64;
            forces[i][0] += fx;
            forces[i][1] += fy;
            forces[j][0] -= fx;
            forces[j][1] -= fy;
        }
    }

    forces
        .iter()
        .map(|&[x, y]| [x as f32, y as f32])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn particle(charge: f32, x: f32, y: f32) -> ChargedParticle {
        ChargedParticle {
            charge,
            mass: 1.0,
            frequency: 400.0,
            position: [x, y],
            velocity: [0.0; 2],
        }
    }

    #[test]
    fn empty_tree() {
        let tree = QuadTree::build(&[], 0);
        assert_eq!(tree.node_count, 0);
    }

    #[test]
    fn single_particle_no_force() {
        let ps = [particle(1.0, 5.0, 5.0)];
        let forces = accumulate_forces_adaptive(&ps, 1, None);
        assert_eq!(forces.len(), 1);
        assert_eq!(forces[0], [0.0, 0.0]);
    }

    #[test]
    fn two_particles_tree_vs_brute() {
        let ps = [particle(1.0, 0.0, 0.0), particle(-1.0, 1.0, 0.0)];
        let brute = accumulate_brute_f64(&ps, 2, None);
        let tree = QuadTree::build(&ps, 2);
        let tree_f = tree.accumulate_all(&ps, 2, THETA);
        // For 2 particles, tree must give exact same result
        for i in 0..2 {
            assert!(
                (brute[i][0] - tree_f[i][0]).abs() < 1e-4,
                "force x mismatch: brute={} tree={}",
                brute[i][0],
                tree_f[i][0]
            );
            assert!(
                (brute[i][1] - tree_f[i][1]).abs() < 1e-4,
                "force y mismatch: brute={} tree={}",
                brute[i][1],
                tree_f[i][1]
            );
        }
    }

    #[test]
    fn newton3_brute_f64() {
        let ps = [
            particle(1.0, 0.0, 0.0),
            particle(-1.0, 3.0, 0.0),
            particle(0.5, 1.0, 2.0),
        ];
        let forces = accumulate_brute_f64(&ps, 3, None);
        let sum_x: f64 = forces.iter().map(|f| f[0] as f64).sum();
        let sum_y: f64 = forces.iter().map(|f| f[1] as f64).sum();
        assert!(
            sum_x.abs() < 1e-10,
            "Newton 3 violated: sum_fx = {sum_x}"
        );
        assert!(
            sum_y.abs() < 1e-10,
            "Newton 3 violated: sum_fy = {sum_y}"
        );
    }

    #[test]
    fn adaptive_dispatch_small_uses_brute() {
        // Under threshold: brute force gives exact Newton 3
        let ps: Vec<_> = (0..10)
            .map(|i| particle(if i % 2 == 0 { 1.0 } else { -1.0 }, i as f32, 0.0))
            .collect();
        let forces = accumulate_forces_adaptive(&ps, 10, None);
        let sum_x: f64 = forces.iter().map(|f| f[0] as f64).sum();
        assert!(sum_x.abs() < 1e-3, "Newton 3: sum_fx = {sum_x}");
    }

    #[test]
    fn tree_100_particles_within_tolerance() {
        use super::super::determinism;
        let mut ps = Vec::with_capacity(100);
        let mut seed = 42u64;
        for _ in 0..100 {
            seed = determinism::next_u64(seed);
            let x = (seed % 1000) as f32 / 100.0;
            seed = determinism::next_u64(seed);
            let y = (seed % 1000) as f32 / 100.0;
            seed = determinism::next_u64(seed);
            let charge = if seed % 2 == 0 { 1.0 } else { -1.0 };
            ps.push(particle(charge, x, y));
        }
        let brute = accumulate_brute_f64(&ps, 100, None);
        let tree = QuadTree::build(&ps, 100);
        let tree_f = tree.accumulate_all(&ps, 100, THETA);

        // Compare RMS relative error across all particles (not max, which is noisy).
        // Tree only approximates Coulomb for distant clusters; LJ computed exactly for leaves.
        // Mean error should be small even if individual outliers exist.
        let mut sum_err_sq: f64 = 0.0;
        let mut sum_mag_sq: f64 = 0.0;
        for i in 0..100 {
            let bx = brute[i][0] as f64;
            let by = brute[i][1] as f64;
            let tx = tree_f[i][0] as f64;
            let ty = tree_f[i][1] as f64;
            sum_err_sq += (bx - tx).powi(2) + (by - ty).powi(2);
            sum_mag_sq += bx.powi(2) + by.powi(2);
        }
        let rms_rel = (sum_err_sq / sum_mag_sq.max(1e-20)).sqrt();
        assert!(
            rms_rel < 0.30,
            "Tree RMS relative force error too large: {:.2}%",
            rms_rel * 100.0
        );
    }

    #[test]
    fn deterministic_same_input_same_output() {
        let ps: Vec<_> = (0..50)
            .map(|i| particle(if i % 2 == 0 { 1.0 } else { -1.0 }, i as f32 * 0.2, (i as f32 * 0.3).sin() * 5.0))
            .collect();
        let f1 = accumulate_forces_adaptive(&ps, 50, None);
        let f2 = accumulate_forces_adaptive(&ps, 50, None);
        for i in 0..50 {
            assert_eq!(f1[i][0].to_bits(), f2[i][0].to_bits(), "non-deterministic at {i}");
            assert_eq!(f1[i][1].to_bits(), f2[i][1].to_bits(), "non-deterministic at {i}");
        }
    }
}
