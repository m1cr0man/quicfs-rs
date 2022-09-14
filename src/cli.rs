use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "quicfs")]
#[clap(author = "Lucas S. <lucas@m1cr0man.com>")]
#[clap(bin_name = "quicfs")]
#[clap(about = "Network file system utilising QUIC")]
pub struct QuicFSCli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Client {
        #[clap(short, long, value_parser)]
        server: String,
        #[clap(value_parser)]
        src: Option<String>,
        #[clap(value_parser)]
        dest: Option<String>,
    },
    Server {
        #[clap(short, long, value_parser)]
        listen: String,
        #[clap(short, long, value_parser)]
        serve: Option<String>,
    },
}
