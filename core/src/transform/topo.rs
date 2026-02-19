/// Topological sort for FK dependencies.
use std::collections::{HashMap, HashSet, VecDeque};

use crate::ir::{Table, TableConstraint};

/// Sort tables in dependency order (tables referenced by FKs come first).
/// Falls back to alphabetical order if cycles are detected.
pub fn topological_sort(tables: &mut Vec<Table>) {
    let name_to_idx: HashMap<String, usize> = tables
        .iter()
        .enumerate()
        .map(|(i, t)| (t.name.name.normalized.clone(), i))
        .collect();

    // Build adjacency list: edges from table → tables it depends on
    let mut in_degree: Vec<usize> = vec![0; tables.len()];
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); tables.len()];

    for (i, table) in tables.iter().enumerate() {
        let deps = get_fk_dependencies(table);
        for dep_name in deps {
            if let Some(&dep_idx) = name_to_idx.get(&dep_name) {
                if dep_idx != i {
                    // dep must come before i
                    dependents[dep_idx].push(i);
                    in_degree[i] += 1;
                }
            }
        }
    }

    // Kahn's algorithm
    let mut queue: VecDeque<usize> = VecDeque::new();
    for (i, &deg) in in_degree.iter().enumerate() {
        if deg == 0 {
            queue.push_back(i);
        }
    }

    // Sort queue entries alphabetically for determinism
    let mut sorted_queue: Vec<usize> = queue.into_iter().collect();
    sorted_queue.sort_by(|a, b| {
        tables[*a]
            .name
            .name
            .normalized
            .cmp(&tables[*b].name.name.normalized)
    });
    let mut queue: VecDeque<usize> = sorted_queue.into_iter().collect();

    let mut order: Vec<usize> = Vec::new();

    while let Some(idx) = queue.pop_front() {
        order.push(idx);
        let mut next_ready = Vec::new();
        for &dep in &dependents[idx] {
            in_degree[dep] -= 1;
            if in_degree[dep] == 0 {
                next_ready.push(dep);
            }
        }
        // Sort newly ready nodes alphabetically
        next_ready.sort_by(|a, b| {
            tables[*a]
                .name
                .name
                .normalized
                .cmp(&tables[*b].name.name.normalized)
        });
        queue.extend(next_ready);
    }

    if order.len() == tables.len() {
        // Successful topological sort
        let mut sorted: Vec<Table> = Vec::with_capacity(tables.len());
        for idx in order {
            sorted.push(std::mem::replace(
                &mut tables[idx],
                Table {
                    name: crate::ir::QualifiedName::new(crate::ir::Ident::new("")),
                    columns: vec![],
                    constraints: vec![],
                },
            ));
        }
        *tables = sorted;
    } else {
        // Cycle detected — fall back to alphabetical
        tables.sort_by(|a, b| a.name.name.normalized.cmp(&b.name.name.normalized));
    }
}

/// Extract FK dependency table names from a table.
fn get_fk_dependencies(table: &Table) -> HashSet<String> {
    let mut deps = HashSet::new();

    for constraint in &table.constraints {
        if let TableConstraint::ForeignKey { ref_table, .. } = constraint {
            deps.insert(ref_table.name.normalized.clone());
        }
    }

    for col in &table.columns {
        if let Some(fk) = &col.references {
            deps.insert(fk.table.name.normalized.clone());
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    fn make_table(name: &str, fk_refs: Vec<&str>) -> Table {
        let constraints: Vec<TableConstraint> = fk_refs
            .into_iter()
            .map(|ref_name| TableConstraint::ForeignKey {
                name: None,
                columns: vec![Ident::new("ref_id")],
                ref_table: QualifiedName::new(Ident::new(ref_name)),
                ref_columns: vec![Ident::new("id")],
                on_delete: None,
                on_update: None,
                deferrable: false,
            })
            .collect();

        Table {
            name: QualifiedName::new(Ident::new(name)),
            columns: vec![],
            constraints,
        }
    }

    #[test]
    fn test_no_deps_alphabetical() {
        let mut tables = vec![
            make_table("c", vec![]),
            make_table("a", vec![]),
            make_table("b", vec![]),
        ];
        topological_sort(&mut tables);
        let names: Vec<&str> = tables
            .iter()
            .map(|t| t.name.name.normalized.as_str())
            .collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_simple_dependency() {
        let mut tables = vec![
            make_table("orders", vec!["users"]),
            make_table("users", vec![]),
        ];
        topological_sort(&mut tables);
        let names: Vec<&str> = tables
            .iter()
            .map(|t| t.name.name.normalized.as_str())
            .collect();
        assert_eq!(names[0], "users");
        assert_eq!(names[1], "orders");
    }

    #[test]
    fn test_cycle_falls_back_to_alphabetical() {
        let mut tables = vec![make_table("b", vec!["a"]), make_table("a", vec!["b"])];
        topological_sort(&mut tables);
        let names: Vec<&str> = tables
            .iter()
            .map(|t| t.name.name.normalized.as_str())
            .collect();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn test_chain_dependency() {
        let mut tables = vec![
            make_table("c", vec!["b"]),
            make_table("b", vec!["a"]),
            make_table("a", vec![]),
        ];
        topological_sort(&mut tables);
        let names: Vec<&str> = tables
            .iter()
            .map(|t| t.name.name.normalized.as_str())
            .collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }
}
