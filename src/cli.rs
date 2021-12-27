use clap::clap_app;

pub fn get_app<'a>(version: &'a str) -> clap::App<'a, 'a> {
    let author = "Lucas S. <lucas@m1cr0man.com>";

    clap_app!(app =>
        (name: "QuicFS")
        (version: version)
        (author: author)
        (about: "Network file system utilising QUIC")
        (@subcommand client =>
            (about: "QuicFS Client utility")
            (@arg server: --server -s +required "URI of server")
            (@arg SRC: +required "File to download from server")
            (@arg DEST: +required "Destination file")
        )
        (@subcommand server =>
            (about: "QuicFS Server utility")
            (@arg listen: --listen -l +required "Listening address (addr:port)")
            (@arg serve: --serve -s +required "Directory to serve")
        )
    )
}
