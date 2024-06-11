//! graph stuff for nets
//!
use bevy::{prelude::*, utils::HashSet};

use super::{LineSegment, LineVertex};

/// finds the connected subgraphs using depth first search
fn connected_graphs(
    q_nodes: Query<&LineVertex>,
    q_paths: Query<&LineSegment>,
    q_nodeids: Query<Entity, With<LineVertex>>,
) {
    let mut nodesset = q_nodeids.iter().collect::<HashSet<Entity>>();
    let mut subgraphs: Vec<(Box<[Entity]>, Box<[Entity]>)> = Vec::with_capacity(nodesset.len()); // subgraph(nodes, paths)
    loop {
        let Some(node) = nodesset.iter().next() else {
            break;
        };
        let mut subgraphnodes = HashSet::<Entity>::new();
        let mut subgraphpaths = HashSet::<Entity>::new();
        dfs_recurs(
            &mut subgraphnodes,
            &mut subgraphpaths,
            *node,
            &q_paths,
            &q_nodes,
        );
        nodesset = nodesset
            .difference(&subgraphnodes)
            .map(|x| x.clone())
            .collect();
        subgraphs.push((
            subgraphnodes
                .iter()
                .map(|x| x.clone())
                .collect::<Box<[Entity]>>(),
            subgraphpaths
                .iter()
                .map(|x| x.clone())
                .collect::<Box<[Entity]>>(),
        ));
    }
}

/// every top level call to this function fills visited_nodes and visited_paths with all entities in connected subgraph
fn dfs_recurs(
    visited_nodes: &mut HashSet<Entity>,
    visited_paths: &mut HashSet<Entity>,
    node: Entity,
    q_paths: &Query<&LineSegment>,
    q_nodes: &Query<&LineVertex>,
) {
    if !visited_nodes.insert(node) {
        // already visited this node
        return;
    }
    let lv = q_nodes.get(node).unwrap();
    for branch in &lv.branches {
        if !visited_paths.insert(*branch) {
            // already visited this node
            continue;
        }
        let segment = q_paths.get(*branch).unwrap();
        let next_node = segment.other_vertex(node);
        dfs_recurs(visited_nodes, visited_paths, next_node, q_paths, q_nodes);
    }
}

/*
nodesset
loop:
    let graphnpdes, graphpaths
    dfs_recurs(graphnodes, graphpaths, nodeset.next())
    append graph_nodes
    nodesset = nodesset - graphnodes
    if nodesset.is_empty
        break
*/
