//! graph stuff for nets
//!
use std::collections::HashMap;

use bevy::{prelude::*, utils::hashbrown::HashSet};

use crate::schematic::elements::{readable_idgen::IdTracker, spid::SpId};

use super::{LineSegment, LineVertex};

/*
to assign unique id to each subgraph:
collect &mut to ids for all entities in each subgraph
first, iterate through subgraphs with only 1, use that 1 if not taken. otherwise get new
then for the other subgraphs, find an used id that is not already taken and apply that to whole subgraph, otherwise get new


*/

/// ensures that all nets related elements have SpId component
pub fn insert_spid(
    q: Query<Entity, (Or<(With<LineVertex>, With<LineSegment>)>, Without<SpId>)>,
    mut commands: Commands,
    mut idtracker: ResMut<IdTracker>,
) {
    let netid = SpId::new("", idtracker.new_net_id());
    q.iter().for_each(|e| {
        commands.entity(e).insert(netid.clone());
    })
}

/// finds the connected subgraphs using depth first search
pub fn connected_graphs(
    q_nodes: Query<(&LineVertex, &SpId)>,
    q_paths: Query<(&LineSegment, &SpId)>,
    q_nodeids: Query<Entity, With<LineVertex>>,
    mut commands: Commands,
    mut idtracker: ResMut<IdTracker>,
) {
    let mut nodesset = q_nodeids.iter().collect::<HashSet<Entity>>();
    let mut subgraphs: Vec<Vec<(Entity, String)>> = Vec::with_capacity(nodesset.len()); // subgraph(nodes, paths)
    loop {
        let Some(node) = nodesset.iter().next() else {
            break;
        };
        let mut subgraphnodes = HashMap::<Entity, String>::new();
        let mut subgraphpaths = HashMap::<Entity, String>::new();
        dfs_recurs(
            &mut subgraphnodes,
            &mut subgraphpaths,
            *node,
            &q_paths,
            &q_nodes,
        );
        nodesset = nodesset
            .difference(&subgraphnodes.keys().cloned().collect())
            .map(|x| x.clone())
            .collect();
        subgraphs.push(
            subgraphnodes
                .into_iter()
                .chain(subgraphpaths.into_iter())
                .map(|x| x.clone())
                .collect::<Vec<(Entity, String)>>(),
        );
    }

    let mut ids_taken = HashSet::<String>::new();
    for subgraph in subgraphs.iter() {
        let idset = subgraph
            .iter()
            .map(|x| x.1.clone())
            .collect::<HashSet<String>>();
        let newid;
        if idset.len() == 1 {
            if ids_taken.insert(idset.into_iter().next().unwrap()) {
                // this subgraph needs no modification
                continue;
            } else {
                // give this subgraph a new id
                newid = idtracker.new_net_id();
                ids_taken.insert(newid.clone());
            }
        } else {
            // find the "lowest" of ids and assigning to entire subgraph
            let mut a = idset.into_iter().collect::<Vec<String>>();
            a.sort();
            let mut used_ids = a.into_iter();
            loop {
                let tentative = used_ids
                    .next()
                    .or_else(|| Some(idtracker.new_net_id()))
                    .unwrap();
                if ids_taken.insert(tentative.clone()) {
                    newid = tentative;
                    break;
                }
            }
        }
        subgraph.iter().for_each(|(e, _s)| {
            commands.entity(*e).insert(SpId::new("", newid.clone()));
        });
    }
}

/// every top level call to this function fills visited_nodes and visited_paths with all entities in connected subgraph
fn dfs_recurs(
    visited_nodes: &mut HashMap<Entity, String>,
    visited_paths: &mut HashMap<Entity, String>,
    node: Entity,
    q_paths: &Query<(&LineSegment, &SpId)>,
    q_nodes: &Query<(&LineVertex, &SpId)>,
) {
    let (lv, vid) = q_nodes.get(node).unwrap();
    if visited_nodes.insert(node, vid.get_id().into()).is_some() {
        // already visited this node
        return;
    }
    for branch in &lv.branches {
        let (segment, vid) = q_paths.get(*branch).unwrap();
        if visited_paths.insert(*branch, vid.get_id().into()).is_some() {
            // if !visited_paths.insert(*branch) {
            // already visited this node
            continue;
        }
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
