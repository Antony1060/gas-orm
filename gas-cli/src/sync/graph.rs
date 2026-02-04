use crate::error::{GasCliError, GasCliResult};
use crate::manifest::GasManifest;
use crate::sync::{FieldDependency, FieldState, ModelChangeActor};
use std::borrow::Cow;
use std::collections::HashMap;

// graph[i] will contain indices of dependencies for the i-th element of the original graph
// NOTE: this system may have a problem if there's 2 actions that both want a field Existing
//  but one of them makes it Dropped, they may be ordered wrong and the field may be dropped
//  and then accessed by the other field; this should not be a problem though by the
//  nature of migrations, if a field is dropped, there won't be diffs that do something with it
fn make_graph<'a>(
    manifest: &GasManifest,
    diffs: &[Box<dyn ModelChangeActor + 'a>],
) -> GasCliResult<Vec<Vec<usize>>> {
    let mut graph = vec![vec![]; diffs.len()];

    let mut map: HashMap<FieldDependency, Vec<usize>> = HashMap::new();

    // make sure all previous fields are here
    for (table_name, fields) in &manifest.state {
        for field in fields {
            map.insert(
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
            let Some(val) = map.get(&field) else {
                return Err(GasCliError::MigrationsGenerationError {
                    reason: Cow::from("failed to change graph: required dependency missing"),
                });
            };

            graph[index].extend(val)
        }
    }

    Ok(graph)
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
) -> GasCliResult<Box<[Box<dyn ModelChangeActor + 'a>]>> {
    let graph = make_graph(manifest, diffs)?;

    Ok(topological_sort(diffs, &graph))
}
