
use openusd_rs::{usd, sdf, tf::Token};
use std::collections::HashSet;

// Prune any prim whose path contains a segment named "Prototypes"
fn is_in_prototypes_subtree(path: &sdf::Path) -> bool {
    let s = path.to_string();
    s.contains("/Prototypes/") || s.ends_with("/Prototypes")
}

// Identify PointInstancer prims using Token comparison.
fn is_point_instancer(prim: &usd::Prim) -> bool {
    prim.type_name() == Token::new("PointInstancer")
}

// Traverse once and collect both: leaf prims and point instancers.
// - Leaves are prims with no children after pruning Prototypes (and we don't count instancers as leaves).
// - Instancers are recorded and we do not descend into them.
fn collect_leaves_and_instancers(stage: &usd::Stage) -> (Vec<sdf::Path>, Vec<sdf::Path>) {
    let root = stage.pseudo_root();

    // Start with root's direct children
    let mut stack: Vec<usd::Prim> = root.children().collect();

    let mut leaves: Vec<sdf::Path> = Vec::new();
    let mut instancers: Vec<sdf::Path> = Vec::new();

    // Optional: prevent duplicates if the stage has variant-induced overlaps
    let mut seen_inst: HashSet<sdf::Path> = HashSet::new();
    let mut seen_leaf: HashSet<sdf::Path> = HashSet::new();

    while let Some(prim) = stack.pop() {
        let p = prim.path().clone();

        // 1) Prune any /.../Prototypes/... subtree
        if is_in_prototypes_subtree(&p) {
            continue;
        }

        // 2) If this is a PointInstancer: record and DO NOT descend
        if is_point_instancer(&prim) {
            if seen_inst.insert(p.clone()) {
                instancers.push(p);
            }
            continue;
        }

        // 3) Otherwise, collect owned child paths (post-pruning) to avoid borrowing `prim`
        let child_paths: Vec<sdf::Path> = prim
            .children()
            .map(|c| c.path().clone())
            .filter(|cp| !is_in_prototypes_subtree(cp))
            .collect();

        if child_paths.is_empty() {
            if seen_leaf.insert(p.clone()) {
                leaves.push(p);
            }
            continue;
        }

        // 4) Reacquire children from the stage and continue traversal
        for cp in child_paths {
            let child_prim = stage.prim_at_path(cp);
            stack.push(child_prim);
        }
    }

    (leaves, instancers)
}

fn main() {
    // Hardcoded USD file path (adjust if needed)
    let path = "C:/Users/Nicol/dev/rust/usd/monkeysUSD/Helmet_bus.usdc";

    let stage = usd::Stage::open(path);

    let (leaves, instancers) = collect_leaves_and_instancers(&stage);

    println!("Leaf prims (excluding /Prototypes and instancers): {}", leaves.len());
    for p in &leaves {
        println!("{p}");
    }

    println!();
    println!("PointInstancers: {}", instancers.len());
    for p in &instancers {
        println!("{p}");
    }
}

