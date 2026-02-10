use crate::error::{GasCliError, GasCliResult};
use crate::manifest::GasManifest;
use crate::sync::{FieldDependency, FieldState, ModelChangeActor};
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};

struct Graph {
    // incoming[i] has i's dependencies
    incoming: Vec<Vec<usize>>,
    // outgoing[i] has nodes that depend on i
    outgoing: Vec<Option<Vec<usize>>>,
}

// graph[i] will contain indices of dependencies for the i-th element of the original graph
// NOTE: this system may have a problem if there's 2 actions that both want a field Existing
//  but one of them makes it Dropped, they may be ordered wrong and the field may be dropped
//  and then accessed by the other field; this should not be a problem though by the
//  nature of migrations, if a field is dropped, there won't be diffs that do something with it
fn make_graph<'a>(
    manifest: &GasManifest,
    diffs: &[Box<dyn ModelChangeActor + 'a>],
) -> GasCliResult<Graph> {
    let mut graph = Graph {
        incoming: vec![vec![]; diffs.len()],
        outgoing: vec![Some(vec![]); diffs.len()],
    };

    let mut provides_map: HashMap<FieldDependency, Vec<usize>> = HashMap::new();

    // make sure all previous fields are here
    for (table_name, fields) in &manifest.state {
        for field in fields {
            provides_map.insert(
                FieldDependency {
                    table_name,
                    name: field.name.as_ref(),
                    state: FieldState::Existing,
                },
                vec![],
            );
        }
    }

    for (index, diff) in diffs.iter().enumerate() {
        for field in diff.provides() {
            provides_map.entry(field).or_default().push(index);
        }
    }

    for (index, diff) in diffs.iter().enumerate() {
        for field in diff.depends_on() {
            match provides_map.get(&field) {
                Some(val) if field.state == FieldState::Existing => {
                    // current node depends on all nodes that provide this field
                    graph.incoming[index].extend(val);

                    // all nodes that provide this field are depended on by current node
                    for it in val {
                        if let Some(vec) = graph.outgoing[*it].as_mut() {
                            vec.push(index)
                        }
                    }
                }
                // NOTE: heavy tight coupling
                //  diffs that depend on a field being Existing will have an inverse of it's action
                //  where all fields that they depend on Existing will now need to be Dropped
                //  which doesn't make much sense because the field that drops the field should
                //  depend on the current diff, so we swap
                // i.e. all providers depend on current index
                Some(val) if field.state == FieldState::InverseDropped => {
                    if let Some(it) = graph.outgoing[index].as_mut() {
                        it.extend(val)
                    }

                    for it in val {
                        graph.incoming[*it].push(index)
                    }
                }
                None if field.state == FieldState::InverseDropped => {}
                _ => {
                    return Err(GasCliError::MigrationsGenerationError {
                        reason: Cow::from("failed to change graph: required dependency missing"),
                    });
                }
            }
        }
    }

    Ok(graph)
}

fn topological_sort<'a>(
    diffs: Vec<Box<dyn ModelChangeActor + 'a>>,
    mut graph: Graph,
) -> GasCliResult<Box<[Box<dyn ModelChangeActor + 'a>]>> {
    let mut diffs: Vec<_> = diffs.into_iter().map(Some).collect();

    let mut sorted: Vec<Box<dyn ModelChangeActor + 'a>> = Vec::new();
    let mut edgeless: VecDeque<usize> = graph
        .incoming
        .iter()
        .enumerate()
        .filter_map(|(idx, it)| it.is_empty().then_some(idx))
        .collect();

    // go through everything without a dependency
    while let Some(diff_idx) = edgeless.pop_front() {
        // add to sorted and remove from original diffs vector
        let curr = diffs[diff_idx].take().expect("diffs[idx] == Some");
        sorted.push(curr);

        // go through everything that depends on idx
        let outgoing = graph.outgoing[diff_idx]
            .take()
            .expect("graph.outgoing[idx] == Some");
        for edge in outgoing {
            // remove edge as a dependency of idx
            let incoming_idx = graph.incoming[edge]
                .iter()
                .position(|it| *it == diff_idx)
                .unwrap();
            graph.incoming[edge].swap_remove(incoming_idx);

            // if idx was the only dependency
            if graph.incoming[edge].is_empty() {
                edgeless.push_back(edge);
            }
        }
    }

    if !graph.outgoing.iter().all(|it| it.is_none()) {
        return Err(GasCliError::MigrationsGenerationError {
            reason: Cow::from("failed to sort diff graph"),
        });
    }

    Ok(sorted.into_boxed_slice())
}

pub fn order_diffs<'a>(
    manifest: &GasManifest,
    diffs: Vec<Box<dyn ModelChangeActor + 'a>>,
) -> GasCliResult<Box<[Box<dyn ModelChangeActor + 'a>]>> {
    let graph = make_graph(manifest, &diffs)?;

    let sorted = topological_sort(diffs, graph)?;

    Ok(sorted)
}
