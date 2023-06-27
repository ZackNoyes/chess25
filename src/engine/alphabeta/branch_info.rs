use std::ops::{Index, IndexMut};

/// This represents the information for a certain depth.
/// - `not_pruned` is the number of nodes that were actually searched at a
///   certain depth.
///   - `expanded` is the number of nodes (of the `not_pruned` nodes) that were
///      actually expanded (rather than being resolved by a table lookup).
///  - `pruned` is the number of nodes that were never searched for a given
///    depth, because the were pruned.
#[derive(Clone, Copy)]
pub struct LayerInfo {
    pub not_pruned: u64,
    pub expanded: u64,
    pub pruned: u64,
}
impl LayerInfo {
    pub fn new() -> Self {
        LayerInfo {
            not_pruned: 0,
            expanded: 0,
            pruned: 0,
        }
    }
}

pub struct BranchInfo(Vec<LayerInfo>);

impl Index<usize> for BranchInfo {
    type Output = LayerInfo;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for BranchInfo {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl BranchInfo {
    pub fn new(depth: u8) -> Self {
        BranchInfo(vec![LayerInfo::new(); depth as usize + 1])
    }

    pub fn statistics(&self) -> String {
        let mut s = String::new();
        
        s.push_str("Pruning statistics:\n");

        for depth in (0..self.0.len() as usize).rev() {
        
            let d = self.0.len() as usize - depth - 1;

            let np = self.0[depth].not_pruned;
            let p = self.0[depth].pruned;
            let e = self.0[depth].expanded;
            let t = np + p;
            let l = np - e;

            if depth == self.0.len() as usize - 1 {
                s.push_str(&format!("\tDepth {} (root) had {} nodes:\n",
                    d, t));
            } else {
                s.push_str(&format!("\tDepth {} had {} nodes (avg. branching factor of {}):\n",
                    d, t, t.checked_div(self.0[depth + 1].expanded).unwrap_or(0)));
            }

            s.push_str(&format!("\t\t{} ({}%) were expanded\n",
                e, (e * 100).checked_div(t).unwrap_or(0)));
            s.push_str(&format!("\t\t{} ({}%) were resolved with a table lookup\n",
                l, (l * 100).checked_div(t).unwrap_or(0)));
            s.push_str(&format!("\t\t{} ({}%) were pruned\n",
                p, (p * 100).checked_div(t).unwrap_or(0)));
        }
        
        s
    }

    pub fn reset_statistics(&mut self) {
        self.0 = vec![LayerInfo::new(); self.0.len() as usize];
    }
}