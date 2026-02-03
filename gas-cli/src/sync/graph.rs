use crate::manifest::GasManifest;
use crate::sync::{FieldDependency, ModelChangeActor};
use std::collections::HashMap;

// graph[i] will contain indices of dependencies for the i-th element of the original graph
fn make_graph<'a>(
    manifest: &GasManifest,
    diffs: &[Box<dyn ModelChangeActor + 'a>],
) -> Vec<Vec<usize>> {
    let mut graph = vec![vec![]; diffs.len()];

    let mut map: HashMap<FieldDependency, Vec<usize>> = HashMap::new();

    for (index, diff) in diffs.iter().enumerate() {
        for field in diff.provides() {
            match map.get_mut(&field) {
                None => {
                    map.insert(field, vec![index]);
                }
                Some(val) => {
                    val.push(index);
                }
            };
        }
    }

    for (index, diff) in diffs.iter().enumerate() {
        for field in diff.depends_on() {
            if let Some(val) = map.get(&field) {
                graph[index].extend(val)
            }
        }
    }

    graph
}

fn topological_sort<'a>(
    diffs: &[Box<dyn ModelChangeActor + 'a>],
    graph: &[Vec<usize>],
) -> Box<[Box<dyn ModelChangeActor + 'a>]> {
    todo!()
}

pub fn order_diffs<'a>(
    manifest: &GasManifest,
    diffs: &[Box<dyn ModelChangeActor + 'a>],
) -> Box<[Box<dyn ModelChangeActor + 'a>]> {
    let graph = make_graph(manifest, diffs);

    topological_sort(diffs, &graph)
}
