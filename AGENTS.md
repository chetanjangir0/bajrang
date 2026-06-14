## Project info
This project is a modular structural analysis engine written in Rust
using the stiffness matrix method.


## Philosophy

This project prioritizes:

* Correctness
* Clean architecture
* Strong engineering abstractions
* Maintainability
* Extensibility
* Separation of concerns

Design decisions should favor long-term scalability over short-term convenience.

---

# General Rules

* Keep modules focused and cohesive.
* Prefer composition over tightly coupled logic.
* Avoid premature optimization.
* Avoid unnecessary abstractions until patterns are stable.
* Prefer explicit and readable code over clever code.
* Keep APIs small and predictable.
* Use strong typing wherever possible.
* Avoid global mutable state.
* Minimize hidden side effects.
* Avoid using excessive code commnents & also for self-explanatory parts 
* Suggest a good commit message after any work you do at the end.

---

# Architecture Principles

* Separate model definition from numerical analysis.
* Separate solver logic from rendering/UI.
* Separate IO/serialization from internal data structures.
* Keep mathematical formulations isolated from application logic.
* Use traits where behavior is shared and stable.
* Prefer immutable data flow when practical.

---

---
# CORE GUI DESIGN DIRECTIVES

1. CLARITY FIRST
Every element must serve a purpose. If you cannot justify why a component exists, remove it. Never add decoration that competes with content.

2. DESIGN FOR THE USER'S MENTAL MODEL
Map the interface to how users think about the task — not how the system works internally. A user wants to "send money," not "POST /api/v2/transactions."

3. ENFORCE HIERARCHY
Use size, weight, and contrast to communicate importance. The most critical action on any screen must be the most visually dominant element.

4. APPLY COGNITIVE LOAD LAWS
- Hick's Law: Fewer options = faster decisions. Default to progressive disclosure.
- Fitts's Law: Frequent actions must be large and close to the user's current focus.
- 80/20 Rule: Design for the 20% of features used 80% of the time. Hide the rest.

5. MAINTAIN CONSISTENCY
Identical actions must look and behave identically across all screens. Do not invent new patterns when an existing one works.

6. ALWAYS PROVIDE FEEDBACK
Every user action must produce a visible response: hover states, loading indicators, success/error messages. Silence is an error state.

7. PREVENT ERRORS BEFORE RECOVERING FROM THEM
Disabled states, confirmation dialogs, and constraints beat undo buttons.

8. ACCESSIBILITY IS NON-NEGOTIABLE
Minimum 4.5:1 contrast ratio. All interactions must be keyboard-navigable. All images need alt text. Treat this as a hard constraint, not a post-launch task.

9. RESPECT PLATFORM CONVENTIONS
Follow the platform's Human Interface Guidelines or Material Design spec unless there is a specific, justified reason to deviate.

10. HANDLE EMPTY STATES
Every list, table, or dashboar

## Iced specific design directives:

Before modifying code that uses Iced toolkit, 
consult below documentation resources:

- api reference
https://docs.rs/iced/latest/iced/

- pocket guide 
https://docs.rs/iced/latest/iced/#the-pocket-guide

- official examples
https://github.com/iced-rs/iced/tree/latest/examples

- official book
https://book.iced.rs/

- unofficial book
https://jl710.github.io/iced-guide/

- How to structure large application?
A common pattern is to leverage this composability to split an application into
different screens:

```rs
use contacts::Contacts;
use conversation::Conversation;

use iced::{Element, Task};

struct State {
    screen: Screen,
}

enum Screen {
    Contacts(Contacts),
    Conversation(Conversation),
}

enum Message {
   Contacts(contacts::Message),
   Conversation(conversation::Message)
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Contacts(message) => {
            if let Screen::Contacts(contacts) = &mut state.screen {
                let action = contacts.update(message);

                match action {
                    contacts::Action::None => Task::none(),
                    contacts::Action::Run(task) => task.map(Message::Contacts),
                    contacts::Action::Chat(contact) => {
                        let (conversation, task) = Conversation::new(contact);

                        state.screen = Screen::Conversation(conversation);

                        task.map(Message::Conversation)
                    }
                 }
            } else {
                Task::none()    
            }
        }
        Message::Conversation(message) => {
            if let Screen::Conversation(conversation) = &mut state.screen {
                conversation.update(message).map(Message::Conversation)
            } else {
                Task::none()    
            }
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    match &state.screen {
        Screen::Contacts(contacts) => contacts.view().map(Message::Contacts),
        Screen::Conversation(conversation) => conversation.view().map(Message::Conversation),
    }
}
```
The update method of a screen can return an Action enum that can be leveraged by
the parent to execute a task or transition to a completely different screen
altogether. The variants of Action can have associated data. For instance, in
the example above, the Conversation screen is created when Contacts::update
returns an Action::Chat with the selected contact. Effectively, this approach
lets you “tell a story” to connect different screens together in a type safe
way. Furthermore, functor methods like Task::map, Element::map, and
Subscription::map make composition seamless.


---

# Numerical Design

* Prioritize numerical correctness and stability.
* Keep coordinate transformations explicit.
* Avoid hardcoded assumptions tied to specific element types.
* Keep local and global systems clearly separated.
* Use deterministic assembly and solving procedures.

---

# Development Practices

* Implement incrementally.
* Keep commits small and focused.
* Write tests alongside implementations.
* Always prefer using verified, solved & established tests
  (also there shouldn't be any license conflict for test)
* THE test cases should be numerically correct.
* Validate against known analytical solutions whenever possible.
* Refactor only after behavior is verified.

---

# Rust Practices

* Prefer enums and typed structures over magic values.
* Prefer Result-based error handling.
* Avoid unwrap in library code.
* Keep ownership semantics clear.
* Keep public APIs minimal.
* Derive traits only when meaningful.

---

# Long-Term Direction

The architecture should remain extensible toward:

* Multiple element types
* Sparse solvers
* Parallel assembly
* Advanced analysis methods
* Visualization systems
* GUI integration
* CAD/BIM interoperability


### IMPORTANT NOTES:

the stifffness matrix defined should be humanly readable eg
#[rustfmt::skip]
-        let k = DMatrix::from_row_slice(4, 4, &[
-             c*c,  c*s, -c*c, -c*s,
-             c*s,  s*s, -c*s, -s*s,
-            -c*c, -c*s,  c*c,  c*s,
-            -c*s, -s*s,  c*s,  s*s,
-        ]);
(try to preserve the #[rustfmt::skip]) for mathmatical expressions


# folder and file structure the project should follow (
whenever the need for extension)

bajrang/
│
├── Cargo.toml                         # workspace definition
├── Cargo.lock
├── README.md
├── LICENSE
├── .gitignore
│
├── docs/                              # theory, derivations, architecture
│   ├── fem/
│   │   ├── truss2d.md
│   │   ├── beam2d.md
│   │   ├── frame2d.md
│   │   └── transformations.md
│   │
│   ├── architecture/
│   │   ├── solver_pipeline.md
│   │   ├── dof_system.md
│   │   └── assembly.md
│   │
│   └── roadmap.md
│
├── examples/                          # sample models
│   ├── truss2d/
│   │   ├── cantilever.json
│   │   └── bridge.json
│   │
│   ├── frame2d/
│   │   └── portal_frame.json
│   │
│   └── beam2d/
│       └── simply_supported.json
│
├── assets/
│   ├── images/
│   └── fonts/
│
├── tests/                             # integration tests
│   └── regression.rs
│
├── benches/
│   ├── assembly.rs
│   ├── sparse_solver.rs
│   └── large_models.rs
│
├── tools/
│   ├── mesh_converter/
│   └── dxf_importer/
│
├── crates/
│
│   ├── math/                          # low-level numerical operations
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       │
│   │       ├── matrix/
│   │       │   ├── dense.rs
│   │       │   ├── sparse.rs
│   │       │   ├── skyline.rs
│   │       │   └── csr.rs
│   │       │
│   │       ├── vector/
│   │       │   ├── dense.rs
│   │       │   └── operations.rs
│   │       │
│   │       ├── decomposition/
│   │       │   ├── cholesky.rs
│   │       │   ├── lu.rs
│   │       │   ├── qr.rs
│   │       │   └── eigen.rs
│   │       │
│   │       ├── iterative/
│   │       │   ├── cg.rs
│   │       │   ├── gmres.rs
│   │       │   └── preconditioner.rs
│   │       │
│   │       └── utils/
│   │           ├── norms.rs
│   │           └── tolerance.rs
│   │
│   │
│   ├── model/                         # structural model definitions
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       │
│   │       ├── geometry/
│   │       │   ├── node.rs
│   │       │   ├── coordinate_system.rs
│   │       │   └── transform.rs
│   │       │
│   │       ├── materials/
│   │       │   ├── material.rs
│   │       │   ├── steel.rs
│   │       │   └── concrete.rs
│   │       │
│   │       ├── sections/
│   │       │   ├── section.rs
│   │       │   ├── rectangular.rs
│   │       │   ├── circular.rs
│   │       │   └── i_section.rs
│   │       │
│   │       ├── loads/
│   │       │   ├── nodal_load.rs
│   │       │   ├── distributed_load.rs
│   │       │   ├── thermal_load.rs
│   │       │   └── load_case.rs
│   │       │
│   │       ├── boundary/
│   │       │   ├── support.rs
│   │       │   ├── constraint.rs
│   │       │   └── releases.rs
│   │       │
│   │       ├── dof/
│   │       │   ├── dof.rs
│   │       │   ├── dof_map.rs
│   │       │   └── numbering.rs
│   │       │
│   │       └── elements/
│   │           ├── mod.rs
│   │           ├── traits.rs
│   │           │
│   │           ├── truss/
│   │           │   ├── truss2d.rs
│   │           │   └── truss3d.rs
│   │           │
│   │           ├── beam/
│   │           │   ├── beam2d.rs
│   │           │   └── beam3d.rs
│   │           │
│   │           ├── frame/
│   │           │   ├── frame2d.rs
│   │           │   └── frame3d.rs
│   │           │
│   │           ├── shell/
│   │           │   ├── quad4.rs
│   │           │   └── tri3.rs
│   │           │
│   │           └── solid/
│   │               ├── tetra4.rs
│   │               └── hexa8.rs
│   │
│   │
│   ├── solver/                        # equation solving systems
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       │
│   │       ├── direct/
│   │       │   ├── cholesky.rs
│   │       │   ├── lu.rs
│   │       │   └── sparse_ldlt.rs
│   │       │
│   │       ├── iterative/
│   │       │   ├── cg.rs
│   │       │   ├── bicgstab.rs
│   │       │   └── gmres.rs
│   │       │
│   │       ├── eigen/
│   │       │   ├── lanczos.rs
│   │       │   └── subspace.rs
│   │       │
│   │       └── nonlinear/
│   │           ├── newton_raphson.rs
│   │           └── arc_length.rs
│   │
│   │
│   ├── bajrang-core/                          # FEM engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       │
│   │       ├── assembler/
│   │       │   ├── global_stiffness.rs
│   │       │   ├── load_vector.rs
│   │       │   ├── mass_matrix.rs
│   │       │   ├── geometric_stiffness.rs
│   │       │   └── boundary_conditions.rs
│   │       │
│   │       ├── analysis/
│   │       │   ├── linear_static.rs
│   │       │   ├── nonlinear_static.rs
│   │       │   ├── modal.rs
│   │       │   ├── buckling.rs
│   │       │   ├── harmonic.rs
│   │       │   └── transient.rs
│   │       │
│   │       ├── post/
│   │       │   ├── displacements.rs
│   │       │   ├── reactions.rs
│   │       │   ├── stresses.rs
│   │       │   ├── strains.rs
│   │       │   ├── element_forces.rs
│   │       │   └── envelopes.rs
│   │       │
│   │       ├── mesh/
│   │       │   ├── connectivity.rs
│   │       │   ├── adjacency.rs
│   │       │   └── partitioning.rs
│   │       │
│   │       ├── state/
│   │       │   ├── analysis_state.rs
│   │       │   └── solution_state.rs
│   │       │
│   │       └── pipeline/
│   │           ├── preprocess.rs
│   │           ├── solve.rs
│   │           └── postprocess.rs
│   │
│   │
│   ├── io/                            # import/export
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       │
│   │       ├── json/
│   │       │   ├── reader.rs
│   │       │   └── writer.rs
│   │       │
│   │       ├── toml/
│   │       │   ├── reader.rs
│   │       │   └── writer.rs
│   │       │
│   │       ├── yaml/
│   │       │   ├── reader.rs
│   │       │   └── writer.rs
│   │       │
│   │       ├── dxf/
│   │       │   └── importer.rs
│   │       │
│   │       ├── ifc/
│   │       │   └── importer.rs
│   │       │
│   │       ├── results/
│   │       │   ├── export_json.rs
│   │       │   └── export_csv.rs
│   │       │
│   │       └── traits/
│   │           ├── reader.rs
│   │           └── writer.rs
│   │
│   │
│   ├── visualization/                 # rendering + plotting
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       │
│   │       ├── render/
│   │       │   ├── mesh_renderer.rs
│   │       │   ├── wireframe.rs
│   │       │   ├── deformed_shape.rs
│   │       │   └── stress_contours.rs
│   │       │
│   │       ├── camera/
│   │       │   ├── orbit.rs
│   │       │   └── projection.rs
│   │       │
│   │       └── plots/
│   │           ├── shear_force.rs
│   │           ├── bending_moment.rs
│   │           └── mode_shapes.rs
│   │
│   │
│   ├── cli/                           # command line interface
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── commands/
│   │       │   ├── solve.rs
│   │       │   ├── validate.rs
│   │       │   └── export.rs
│   │       │
│   │       └── config/
│   │           └── cli_config.rs
│   │
│   │
│   └── gui/                           # future iced GUI
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── app.rs
│           │
│           ├── state/
│           │   ├── app_state.rs
│           │   ├── viewport_state.rs
│           │   └── selection_state.rs
│           │
│           ├── viewport/
│           │   ├── canvas.rs
│           │   ├── interaction.rs
│           │   ├── picking.rs
│           │   └── gizmos.rs
│           │
│           ├── panels/
│           │   ├── properties.rs
│           │   ├── model_tree.rs
│           │   ├── loads.rs
│           │   ├── supports.rs
│           │   └── analysis.rs
│           │
│           ├── tools/
│           │   ├── draw_node.rs
│           │   ├── draw_member.rs
│           │   └── assign_load.rs
│           │
│           ├── renderer/
│           │   ├── scene.rs
│           │   ├── grid.rs
│           │   ├── members.rs
│           │   └── results.rs
│           │
│           └── theme/
│               └── colors.rs
│
└── .github/
    └── workflows/
        ├── ci.yml
        └── release.yml
