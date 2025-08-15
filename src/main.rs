use openusd_rs::{
    sdf, usd,
    gf::Matrix4d,
    tf::Token
};
use std::collections::HashSet;

fn is_in_prototypes_subtree(path: &sdf::Path) -> bool {
    let s = path.to_string();
    s.contains("/Prototypes/") || s.ends_with("/Prototypes")
}

fn is_point_instancer(prim: &usd::Prim) -> bool {
    prim.type_name() == Token::new("PointInstancer")
}

fn collect_leaves_and_instancers(stage: &usd::Stage) -> (Vec<sdf::Path>, Vec<sdf::Path>) {
    let root = stage.pseudo_root();
    let mut stack: Vec<usd::Prim> = root.children().collect();
    let mut leaves: Vec<sdf::Path> = Vec::new();
    let mut instancers: Vec<sdf::Path> = Vec::new();

    let mut seen_inst: HashSet<sdf::Path> = HashSet::new();
    let mut seen_leaf: HashSet<sdf::Path> = HashSet::new();

    while let Some(prim) = stack.pop() {
        let p = prim.path().clone();

        if is_in_prototypes_subtree(&p) {
            continue;
        }
        if is_point_instancer(&prim) {
            if seen_inst.insert(p.clone()) {
                instancers.push(p);
            }
            continue;
        }

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
        for cp in child_paths {
            let child_prim = stage.prim_at_path(cp);
            stack.push(child_prim);
        }
    }
    (leaves, instancers)
}


fn get_local_transform(prim: &usd::Prim) -> Option<Matrix4d> {
    let tok = Token::new("xformOp:transform");
    if prim.has_attribute(&tok) {
        let attr = prim.attribute(&tok);
        Some(attr.get::<Matrix4d>())
    } else {
        None
    }
}


fn accumulate_transforms(stage: &usd::Stage, start: &usd::Prim) -> Matrix4d {
    let mut total = Matrix4d::identity();
    let mut current: usd::Prim = stage.prim_at_path(start.path().clone());

    loop {
        if let Some(local_xf) = get_local_transform(&current) {
            // child-first accumulation (total = total * local)
            total *= local_xf;
        }
        let parent_path = current.path().parent_path();
        if parent_path.is_empty() { break; }
        current = stage.prim_at_path(parent_path);
    }

    total
}


fn main() {
    let path = "C:/Users/Nicol/dev/rust/usd/descent/Helmet_bus_2.usdc";
    let stage = usd::Stage::open(path);

    let (leaves, instancers) = collect_leaves_and_instancers(&stage);

    println!("Leaf prims (excluding /Prototypes and instancers): {}", leaves.len());
    for p in &leaves {
        let prim = stage.prim_at_path(p.clone());
        let xf = accumulate_transforms(&stage, &prim);
        println!("accumulated_transforms= \n{p} => {:?}", xf);
    }

    println!();
    println!("PointInstancers: {}", instancers.len());
    for p in &instancers {
        let prim = stage.prim_at_path(p.clone());
        let xf = accumulate_transforms(&stage, &prim);
        println!("accumulated_transforms= \n{p} => {:?}", xf);
    }
}
