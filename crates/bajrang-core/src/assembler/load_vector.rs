use model::{
    dof::{DOFS_PER_NODE, global_dof_index},
    elements::traits::Element,
    load::NodalLoad,
    node::Node,
};

/// Assemble the global force vector from nodal and element-equivalent loads.
pub fn assemble_load_vector<E: Element>(
    nodes: &[Node],
    elements: &[E],
    loads: &[NodalLoad],
) -> Vec<f64> {
    let ndof = nodes.len() * DOFS_PER_NODE;
    let mut f = vec![0.0_f64; ndof];

    for load in loads {
        let idx = global_dof_index(load.node_id, load.dof);
        f[idx] += load.magnitude;
    }

    for element in elements {
        let fe = element.equivalent_load_vector(nodes);
        let dofs = element.dof_indices();

        assert_eq!(
            fe.nrows(),
            dofs.len(),
            "Element load vector length must match DOF count",
        );

        for (local_idx, &global_idx) in dofs.iter().enumerate() {
            f[global_idx] += fe[local_idx];
        }
    }

    f
}
