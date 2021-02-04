use log::trace;
use ra_ap_rust_analyzer::cli::load_cargo;
use structopt::StructOpt;

use crate::{
    graph::{
        builder::{Builder as GraphBuilder, Options as GraphBuilderOptions},
        util,
    },
    options::{
        general::Options as GeneralOptions, graph::Options as GraphOptions,
        project::Options as ProjectOptions,
    },
    runner::Runner,
};

pub mod graph;
pub mod tree;

#[derive(StructOpt, Clone, PartialEq, Debug)]
pub enum Command {
    #[structopt(name = "tree", about = "Print crate as a tree.")]
    Tree(tree::Options),

    #[structopt(
        name = "graph",
        about = "Print crate as a graph.",
        after_help = r#"
        If you have xdot installed on your system, you can run this using:
        `cargo modules generate dependencies | xdot -`
        "#
    )]
    Graph(graph::Options),
}

impl Command {
    pub fn run(&self) -> Result<(), anyhow::Error> {
        let general_options = self.general_options();
        let project_options = self.project_options();
        let graph_options = self.graph_options();

        let path = project_options.manifest_dir.as_path();
        let project_path = path.canonicalize()?;

        let (host, vfs) = load_cargo(&project_path, true, false).unwrap();
        let db = host.raw_database();

        let runner = Runner::new(project_path, project_options.to_owned(), db, &vfs);

        runner.run(|krate, package, target| {
            let crate_name = krate.display_name(db).expect("Crate name").to_string();

            if general_options.verbose {
                eprintln!();
                eprintln!("crate: {}", crate_name);
                eprintln!("└── package: {}", package.name);
                eprintln!("    └── target: {}", target.name);
                eprintln!();
            }

            let graph_builder = {
                let builder_options = self.builder_options();
                GraphBuilder::new(builder_options, db, &vfs, krate)
            };

            let focus_path: Vec<_> = {
                let path_string = graph_options.focus_on.clone().unwrap_or(crate_name);
                path_string.split("::").map(|c| c.to_owned()).collect()
            };

            let (graph, start_node_idx) = {
                trace!("Building graph ...");

                let mut graph = graph_builder.build(krate)?;

                trace!("Searching for start node in graph ...");

                let start_node_idx = util::idx_of_node_with_path(&graph, &focus_path[..], db)?;

                trace!("Shrinking graph to desired depth ...");

                let max_depth = graph_options.max_depth.unwrap_or(usize::MAX);
                util::shrink_graph(&mut graph, start_node_idx, max_depth);

                (graph, start_node_idx)
            };

            trace!("Printing ...");

            match &self {
                #[allow(unused_variables)]
                Self::Tree(options) => {
                    let command = tree::Command::new(options.clone());
                    command.run(&graph, start_node_idx, krate, db)
                }
                #[allow(unused_variables)]
                Self::Graph(options) => {
                    let command = graph::Command::new(options.clone());
                    command.run(&graph, start_node_idx, krate, db)
                }
            }
        })
    }

    fn general_options(&self) -> &GeneralOptions {
        match &self {
            Self::Tree(options) => &options.general,
            Self::Graph(options) => &options.general,
        }
    }

    fn project_options(&self) -> &ProjectOptions {
        match &self {
            Self::Tree(options) => &options.project,
            Self::Graph(options) => &options.project,
        }
    }

    fn graph_options(&self) -> &GraphOptions {
        match &self {
            Self::Tree(options) => &options.graph,
            Self::Graph(options) => &options.graph,
        }
    }

    fn builder_options(&self) -> GraphBuilderOptions {
        match &self {
            Self::Tree(options) => GraphBuilderOptions {
                focus_on: options.graph.focus_on.clone(),
                max_depth: options.graph.max_depth,
                with_types: options.graph.with_types,
                with_tests: options.graph.with_tests,
                with_orphans: options.graph.with_orphans,
                with_uses: false,
                with_externs: false,
            },
            Self::Graph(options) => GraphBuilderOptions {
                focus_on: options.graph.focus_on.clone(),
                max_depth: options.graph.max_depth,
                with_types: options.graph.with_types,
                with_tests: options.graph.with_tests,
                with_orphans: options.graph.with_orphans,
                with_uses: options.with_uses,
                with_externs: options.with_externs,
            },
        }
    }
}
